# Tri par comptage des octets (u8, i8)

Notes du 2026-11-18, après lecture de `blazing-radix 0.1.0` (`src/engine.rs` :
`sort_bytes`, `byte_scatter`, `byte_kernel`, `fill_bytes`, `fill_byte_bucket`)
et comparaison avec `v2::engines::counting::domain_count`.

## 0. Le tri par comptage

Un octet ne prend que 256 valeurs. Au lieu de comparer les clés, on les compte.

### Des valeurs seules (u8, i8) : compter, puis réécrire

```
tableau :  3 1 3 0 1 3

1) compter        valeur :  0  1  2  3
                  compte :  1  2  0  3

2) réécrire chaque valeur autant de fois qu'elle a été comptée, dans l'ordre :

   position = 0
   valeur 0, 1 fois :  [0]               position = 1
   valeur 1, 2 fois :  [0 1 1]           position = 3
   valeur 2, 0 fois :  [0 1 1]           position = 3
   valeur 3, 3 fois :  [0 1 1 3 3 3]     position = 6

résultat   :  0 1 1 3 3 3
```

Une **case** = l'emplacement des copies d'une même valeur dans le résultat
(ici : case 0 = position 0, case 1 = positions 1-2, case 3 = positions 3-5).
La position qui avance (`position += compte`) donne le début de chaque case
au moment où on l'écrit : il n'y a pas besoin de les calculer à l'avance.
C'est ce que font notre code et le `byte_kernel` de blazing.

### Déplacer des éléments : compter, cumuler, poser

Quand on ne peut pas se contenter de réécrire des valeurs, il faut le début
de chaque case **avant** de commencer à poser les éléments, d'où une étape
de cumul (somme des comptes des valeurs plus petites) :

```
comptes :  0:1  1:2  2:0  3:3
débuts  :  0:0  1:1  2:3  3:3     début[v] = compte[0] + ... + compte[v - 1]
```

Puis chaque élément lu est posé à `début[sa clé]`, qui avance de 1. C'est
nécessaire pour :

- **des records** (une clé et des données) : on ne peut pas « réécrire trois
  fois la clé 7 », il faut déplacer chaque record entier à sa place ;
- **le tri stable** : les éléments de même clé gardent leur ordre, donc on
  les pose un par un, dans l'ordre de lecture ;
- **la pose via un buffer** (section 3) : on pose chaque clé à sa place au fil
  de la lecture ;
- **les passes d'un tri radix (LSD)** : chaque passe déplace les éléments
  selon un chiffre de leur clé.

### Coût

**n** (lire et compter) + **256** (parcourir les cases du compteur) + **n** (écrire).
Sur un grand tableau, les termes en n dominent et on est limité par la
mémoire : tous les bons tris par comptage se valent. Sur un petit tableau
(quelques centaines de clés), le terme **256** et les frais fixes (préparer
les compteurs, l'appel) pèsent autant que n : c'est là que les écarts se
font.

## 1. Plusieurs tables de compteurs

Incrémenter un compteur, c'est lire la mémoire, ajouter 1, écrire. Si deux
clés qui se suivent ont la même valeur, le second incrément doit attendre que
le premier soit écrit (quelques cycles) : sur des clés répétées, les
incréments s'enchaînent au lieu d'avancer en parallèle.

```
une table :      clés   7  7  7  7  7  7
                        │  │  │  │  │  │
                        ▼  ▼  ▼  ▼  ▼  ▼
                 c[7]  +1→+1→+1→+1→+1→+1      chaque +1 attend le précédent

quatre tables :  clés   7  7  7  7  7  7  7  7
                 table  A  B  C  D  A  B  C  D
                        A[7] +1 ─────────→ +1      4 chaînes indépendantes,
                        B[7]    +1 ─────────→ +1   qui avancent ensemble
                        C[7]       +1 ──────→
                        D[7]          +1 ───→
                 à la fin :  compte[7] = A[7] + B[7] + C[7] + D[7]
```

C'est général (tous les processeurs modernes ont cette latence) ; le nombre
de tables (4) est un réglage. Coût : 4 tables à mettre à zéro et à
additionner (4 × 256 compteurs), qui pèse sur les petits tableaux.

L'ancien v2 n'avait qu'une table sous 512 clés : sur des répétitions (Zipf,
Heavy, Sparse 90 %) à 500 clés, 1,5 à 2 fois plus lent que blazing.

## 2. Compter avec des compteurs d'un octet

Un compteur ordinaire est un `u32` (4 octets) : une table de 256 compteurs
fait 1 Ko, quatre tables 4 Ko. Un compteur d'un octet (`u8`) ne compte que
jusqu'à 255, mais la table ne fait que 256 octets.

```
compteurs u32 :  [    0    ][    1    ] ... [   255   ]     256 × 4 o = 1 Ko
compteurs u8  :  [0][1][2] ... [255]                       256 × 1 o = 256 o
```

Pourquoi c'est possible : blazing ne s'en sert que jusqu'à 256 clés, avec 2
tables qui reçoivent chacune une clé sur deux. Une table voit donc au plus
128 clés : aucun compteur ne peut dépasser 128, il ne déborde jamais.

```
256 clés :  k0 k1 k2 k3 k4 k5 ...
table A  :  k0    k2    k4    ...   (128 clés au plus)
table B  :     k1    k3    k5 ...   (128 clés au plus)
```

Ce que ça gagne : moins de mémoire à mettre à zéro et à additionner (512
octets au lieu de 4 Ko), ce qui compte justement sur les petits tableaux. Au-
delà de 256 clés on repasse à des compteurs `u32` (ou on vide les compteurs
d'un octet dans des grands compteurs avant qu'ils débordent : c'est ce que
fait le tri des bool de la v1, toutes les 255 rondes).

## 3. Recopier les clés à leur place via un buffer sur la pile

Pour moins de 256 clés, il y a plus de cases (256) que de clés : parcourir
les 256 cases pour les remplir coûte plus que les clés elles-mêmes. blazing
fait autrement : il calcule où commence chaque case (cumul), puis il **pose
chaque clé directement à sa place** dans un tableau temporaire, et recopie le
tout d'un coup.

```
tableau :      3 1 3 0 1

comptes :      0:1  1:2  2:0  3:2
débuts  :      0:0  1:1  2:3  3:3          (cumul des comptes)

pose clé par clé (début de sa case, puis +1) :
  3 → position 3   tmp: [ . . . 3 . ]      début[3] = 4
  1 → position 1   tmp: [ . 1 . 3 . ]      début[1] = 2
  3 → position 4   tmp: [ . 1 . 3 3 ]      début[3] = 5
  0 → position 0   tmp: [ 0 1 . 3 3 ]      début[0] = 1
  1 → position 2   tmp: [ 0 1 1 3 3 ]      début[1] = 3

recopie tmp → tableau :  0 1 1 3 3
```

Le travail suit le nombre de clés (n poses), plus le parcours des 256 cases.
Le tableau temporaire (`tmp`) est **sur la pile** : un tableau de taille
fixe (256 octets) dans la fonction, sans allocation. Une allocation sur le
tas (`vec!`) coûte un appel à l'allocateur à chaque tri, ce qui se voit sur
un tableau de 500 clés.

```
pile  : [ tmp : 256 octets ]   réservé en entrant dans la fonction, gratuit
tas   : vec![0; 256]           appel à l'allocateur, mise à zéro, libération
```

Note : pour des octets seuls, la pose n'est pas nécessaire (réécrire les
valeurs suffit, section 4) ; elle sert quand la clé porte autre chose (un
record) ou, chez blazing, à éviter 256 remplissages courts.

## 4. Écriture par bloc fixe, et réécriture des cases écrites en trop

Après le comptage, on réécrit chaque case : la valeur 7 apparaît 3 fois, on
écrit `7 7 7` à partir du début de sa case.

**Écriture exacte** : une boucle ou un `memset` de 3 octets. Sa longueur varie
d'une case à l'autre : test, boucle, branche que le processeur prédit mal,
256 fois.

**Écriture par bloc fixe** : on écrit `7` sur un bloc de taille fixe (64 octets
chez blazing) d'un seul coup, même s'il n'en fallait que 3. Pas de boucle, pas
de longueur variable : une ou quelques écritures vectorielles.

Les octets écrits en trop débordent sur les cases suivantes. Ce n'est pas un
problème : on remplit les cases **dans l'ordre croissant**, donc chaque case
suivante réécrit par-dessus le débordement de la précédente.

```
cases : 3 × 7, puis 5 × 9, puis 2 × 12, ...     début = 100

case 7  : bloc de 7  sur [100 ........................ 163]
          utile :          100 101 102
résultat:                  7   7   7   7 7 7 7 7 7 ...  (le reste : en trop)

case 9  : bloc de 9  sur      [103 ........................ 166]
          utile :              103 104 105 106 107
résultat:                  7 7 7 9 9 9 9 9 9 9 9 ...    (les 7 en trop : écrasés)

case 12 : bloc de 12 sur              [108 ................ 171]
résultat:                  7 7 7 9 9 9 9 9 12 12 12 ... (les 9 en trop : écrasés)

...       la dernière case est écrite exactement : rien ne déborde du tableau
```

Seule condition : le bloc ne doit pas dépasser la fin du tableau
(`début + 64 ≤ n`). Près de la fin, ou pour une grosse case, on revient à
l'écriture exacte.

Le revers : le débordement est du travail perdu. Avec beaucoup de valeurs
différentes et peu de clés (500 clés réparties sur 256 valeurs, 2 par case),
chaque case écrit 64 octets pour en garder 2 : environ 16 Ko écrits pour un
tableau de 500 octets. Mesuré sur la Surface (u8, 500 clés, Aligned) : 1,23
ns par clé contre 0,72 pour l'écriture case par case.

blazing empile des paliers (64, puis 32, puis 16, puis 4 × 64 jusqu'à 256,
puis `memset`), choisis pour les tailles de case fréquentes sur ses machines :
c'est un réglage propre au processeur, pas une règle générale.

## 5. Ce qui est général, ce qui est un réglage

| idée                                              | général ? |
|---------------------------------------------------|-----------|
| pas d'allocation par appel (tables sur la pile)   | oui       |
| pas de passage inutile (ne pas retourner un tableau avant de savoir qu'il est trié à l'envers) | oui |
| plusieurs tables pour les clés répétées           | oui (le nombre : réglage) |
| sauter les blocs de 16 valeurs vides au remplissage | oui (un test par bloc) |
| compteurs d'un octet sur les petits tableaux      | oui (moins de mémoire à préparer) |
| poser les clés via un buffer sur la pile (< 256 clés) | oui (travail en n, pas en 256) |
| bloc fixe qui déborde (16, 32, 64 octets)         | non : réglage de processeur |
| lire 8 clés d'un coup (`u64`) ou une par une      | non : dépend du processeur (Snapdragon : `u64` mesuré le 18/11 ; Ryzen `u64` et Haswell une par une : déduit de l'écart avec blazing, à mesurer) |
