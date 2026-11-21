# Construire un monde maison3d — guide pour un agent

Ce guide s'affiche aussi avec `maison3d guide`.

## Outils

| Commande | Rôle |
|---|---|
| `maison3d schema --brief` | Une ligne par commande : champs (`?` = facultatif) et rôle. **À lire en premier.** |
| `maison3d help <op>` | Détail d'une commande : description, chaque champ, ses bornes et sa valeur par défaut. |
| `maison3d schema` | Schéma JSON complet (31 Ko). |
| `maison3d catalog` | Objets (rangés par catégorie), ouvertures et toits disponibles, avec leurs dimensions réelles et leurs parties colorables. |
| `maison3d search mots [--room R] [--placement P] [--limit N]` | Les objets qui répondent aux mots (français ou anglais, sans souci d'accents ni de pluriel), les plus pertinents d'abord : nom, taille, pose, dégagement, pièces, objets associés. `--room Kitchen` (ou `--room cuisine`) et `--placement AgainstWall` filtrent ; sans mots, liste ce qui passe les filtres. 10 résultats par défaut, `--limit N` pour en voir plus. Trouve les objets, pas les ouvertures ni les toits (voir `catalog`). |
| `maison3d textures` | Numéros des matières (champ `texture`). |
| `maison3d apply monde.json cmds.json [--new] [--quiet] [--dry-run] [--out autre.json] [--json]` | Applique une liste de commandes. |
| `maison3d describe monde.json` | Résumé texte par étage, avec l'indice, le type JSON et l'emprise au sol de chaque élément. |
| `maison3d query monde.json rooms` | Les pièces de chaque étage : l'espace fermé par les murs, cloisons comprises. Nom, un point intérieur, surface, nombre d'objets et de portes. |
| `maison3d query monde.json room <nom ou x,z> [--level N]` | Une pièce : rectangle entre les murs, chaque mur avec son côté (nord, sud, est, ouest) et la coordonnée de sa **face intérieure**, ses ouvertures avec leur centre, ses objets. |
| `maison3d query monde.json near x,z [--level N] [--radius R]` | La pièce qui contient le point, puis murs, ouvertures et objets à moins de R m (2 par défaut), du plus proche au plus loin. |
| `maison3d suggest monde.json <Sorte> --room <nom ou x,z> [--level N] [--facing n ou Sorte] [--count N]` | Places proposées pour un objet dans une pièce, de la meilleure à la moins bonne : chaque ligne est une commande `place_prop` prête à coller, suivie de ses raisons (contre quel mur, tourné vers quoi, à côté de quel objet qui va avec, dégagement libre ou non, fenêtre masquée). Ne marche que dans une pièce fermée (dehors : poser avec `place_prop`). Ne propose que des places dans la pièce, sans traverser de mur ni de cloison, sans chevauchement (même règle que `check` : un objet mural peut passer au-dessus d'un meuble bas), hors du passage des portes et de l'espace à laisser devant les autres meubles ; un objet mural (miroir, radiateur) jamais sur une porte ni une fenêtre. **Ce sont des propositions** : les suivre, les ajuster ou poser ailleurs. |
| `maison3d check monde.json` | Avertissements de cohérence. `apply` les affiche aussi après chaque lot. |
| `maison3d shot monde.json vue.png [options]` | Vue 3/4 (`vue.png`) et plan à l'échelle (`vue_dessus.png`, nord en haut). |

### `apply`

- **Tout ou rien** : à la première commande refusée, le monde n'est pas écrit, le code de sortie
  vaut 1, et le message (sur stderr) donne le JSON de la commande avec la raison du refus.
- **Monde absent** : refusé, sauf avec `--new`.
- **Sortie** : pour chaque commande, `apply` affiche les éléments qu'elle a créés
  (`#3 ok → créés Wall 4 à 7, créé Slab 1`), puis les avertissements de `check`, puis `describe`.
  `--quiet` supprime le `describe`.
- **Essayer sans risque** : `--dry-run` valide et affiche tout sans rien écrire ; `--out autre.json`
  écrit le résultat ailleurs et laisse le monde lu intact (variante jetable).
- **Sortie pour un script** : `--json` remplace le texte par une ligne JSON par commande,
  `{"index":0,"ok":true,"elem":{"Prop":3},"created":[{"Prop":3}],"removed":{}}`, puis une ligne
  `{"saved":"monde.json","warnings":[…]}` (`saved` vaut `null` avec `--dry-run`). Les références
  se recopient telles quelles dans `target`. Un refus donne `{"index":…,"ok":false,"error":"…"}`
  et le code de sortie 1.

### Options de `shot`

| Option | Effet |
|---|---|
| `--level N` | Coupe au-dessus de l'étage N, pour la 3/4 comme pour le plan. Sans elle : 3/4 entière et plan du rez-de-chaussée. |
| `--from so\|se\|no\|ne` | Point de vue de la 3/4. Par défaut `so` : la caméra est au sud-ouest et voit les façades sud et ouest. |
| `--zone x0,z0,x1,z1` | Cadre cette zone. Par défaut le cadrage porte sur les bâtiments (murs, sols, toits), pas sur les routes ni les arbres. |
| `--size LxH` | Taille des images (1280x800 par défaut). |
| `--only 3/4` ou `--only plan` | Une seule image. |

La caméra se place d'elle-même au-dessus du relief. `shot` rend hors écran (aucune fenêtre
ne s'ouvre, en 2 à 3 secondes) et écrit sur la sortie standard le chemin de chaque image produite.

## Boucle de travail

1. Lire `schema --brief` ; `help <op>` au besoin. Pour trouver un objet, `search` plutôt que
   lire tout `catalog` (`search lit`, `search --room Bathroom`).
2. Écrire un lot de commandes cohérent (une pièce et ses ouvertures, un étage…), puis `apply`.
   Pour les répétitions, générer le JSON avec un petit script.
3. Lire la sortie :
   - les indices créés : les noter, plutôt que de les recompter ;
   - les avertissements : les corriger avant de continuer.
4. Avant de meubler une pièce, `query monde.json room <nom>` : les faces intérieures des murs
   et la place des portes y sont données, inutile de les recalculer. Pour un objet dont la place
   n'est pas évidente, `suggest` propose des positions ; poser d'abord les grands meubles
   (lit, canapé, armoire), puis ceux qui vont avec. `--facing` sert aux objets qui **regardent**
   un autre : la table basse ou le fauteuil vers le canapé, la chaise vers le bureau. Pour ceux
   qui vont **à côté** (chevet, lampe) ou **au-dessus** (miroir sur le lavabo), ne pas le donner :
   `suggest` les rapproche déjà des objets avec lesquels ils vont.
5. `shot` pour contrôler à l'œil. Utiliser `--level` pour chaque étage, et `--from` pour voir
   les autres façades.
6. Recommencer avec le lot suivant, sur le même `monde.json`.

Exemples complets et testés dans `docs/exemples/` :

| Fichier | Contenu |
|---|---|
| `maison-simple.json` | Une pièce meublée et son toit. |
| `maison-deux-pieces.json` | Cloison, porte intérieure et pièces nommées. |
| `maison-etage.json` | Étage, trémie et escalier. |
| `rue.json` | Chaussée, trottoirs, passage piéton, lampadaires, voitures garées, et deux maisons face à face dont une à étage (`add_storey` limité à une zone). |
| `jardin.json` | Relief, piscine, allée, route, haie en rangée, lampadaires et végétaux. |
| `retouches.json` | Toutes les commandes `add_*`, `set` et les commandes d'édition. |

## Repère

```
        nord (-z)
          ↑
 ouest ←  ·  → est (+x)
          ↓
        sud (+z)
```

- Un point au sol est `[x, z]` en mètres ; y est la verticale.
- `level` va de -1 à 9. Chaque étage fait 3 m : le sol de `level` n est à 3n m.
- Tout point doit rester sur le terrain : x et z entre -80 et 80.
- Couleurs `[r, g, b]` entre 0 et 1 ; `[1, 1, 1]` montre la texture telle quelle.
- **Valeurs refusées** : hors bornes (épaisseur, hauteur, profondeur… : voir `help`), texture
  inconnue, ou **champ inconnu**. Une faute de frappe comme `rotation` au lieu de `rot` fait
  donc refuser la commande au lieu d'être ignorée.

### Murs et ouvertures

- Un mur va de `a` à `b`. Son épaisseur est centrée sur ce segment : un mur de 20 cm en z = 0
  occupe z de -0,1 à 0,1. La face intérieure d'une pièce est donc à `thick / 2` du bord.
- `room` crée 4 murs dans cet ordre (indices consécutifs) :
  - nord (z = min, de ouest vers est) ;
  - est (x = max, de nord vers sud) ;
  - sud (z = max, de est vers ouest) ;
  - ouest (x = min, de sud vers nord).
  
  `room` accepte `height`, `thick`, `wall_texture` et `floor_texture`.
- `t` d'une ouverture est la distance depuis `a` jusqu'au **centre** de l'ouverture.
  - Sur le mur sud d'une `room` de x 0 à 8, `a` est à x = 8 : `t = 2` place l'ouverture en x = 6.
  - La plage permise laisse 5 cm, plus la demi-épaisseur du mur, à chaque bout. En cas de refus,
    le message donne la plage de `t` valide, ou l'ouverture chevauchée.
- `sill` (allège) est la hauteur du bas de l'ouverture : 0 pour une porte, environ 0,9 m pour une
  fenêtre. `sill + height` ne doit pas dépasser la hauteur du mur.

### Objets

- **Choisir un objet** : chaque entrée de `catalog` dit
  - `rooms` : dans quelles pièces il a sa place ;
  - `placement` : comment le poser (`AgainstWall` = dos au mur, `Floor` = dégagé, `Row` = en rangée…) ;
  - `front` : l'espace à laisser libre devant lui ;
  - `goes_with` : les objets qu'on pose avec lui ;
  - `needs` : ce qu'il exige (une route pour une voiture, une trémie pour un escalier) ;
  - `synonyms` : les mots pour le retrouver.
  
  Pour meubler une chambre, partir des objets dont `rooms` contient `Bedroom`.
- `pos` est le **centre** de l'objet.
- À `rot` 0, l'avant regarde +z (le sud) ; l'avant pointe vers `[sin rot, cos rot]`.
  90 = est, 180 = nord, 270 = ouest.
- `catalog` donne `size = [largeur, profondeur, hauteur]` à `rot` 0. La largeur suit x et la
  profondeur suit z ; à `rot` 90 ou 270, elles s'échangent au sol. `describe` donne l'emprise
  réelle de chaque objet posé.
- Pour coller un objet dos à un mur, utiliser `place_against` : `{"op": "place_against",
  "kind": "Sofa", "wall": 0, "t": 4}` pose le canapé au milieu du mur nord d'une `room` de
  x 0 à 8, arrière contre la face du mur, avant vers la pièce ; `pos` et `rot` sont calculés.
  - `t` est la distance depuis `a` jusqu'au centre de l'objet, comme pour une ouverture ; le
    message de refus donne la plage valide.
  - Le côté est celui où un sol du même étage borde le mur. Sur une cloison (un sol des deux
    côtés) ou un mur sans sol, donner `side` : `Left` ou `Right` en allant de `a` vers `b`
    (`Right` = l'intérieur d'une `room`).
  - À la main : avant tourné vers la pièce, centre à `face du mur + profondeur / 2`. Canapé
    (profondeur 0,9) contre un mur nord de 20 cm en z = 0 → `rot` 0, `pos.z` = 0,1 + 0,45 = 0,55.
- Lit : la tête est à l'arrière, contre le mur. Chaise : tournée vers la table.
- `catalog` range les objets par `category` (Salon, Cuisine, Mobilier urbain, Sport…). Pour un
  décor, parcourir la catégorie voulue plutôt que deviner un nom.
- Les objets `outdoor` du rez-de-chaussée se posent sur le relief.
- `dim` dit ce que règle le champ `height` d'un objet : la hauteur d'un escalier ou d'un pilier,
  la longueur d'une ligne, le rayon d'un cercle. `null` : le champ ne sert pas. Il vaut 3 m par
  défaut (un étage), 0,5 m pour `StoneSteps` et `Ramp`.
- `scale` (0,2 à 5, 1 par défaut) agrandit ou réduit tout l'objet, `height` comprise : arbre
  majestueux, tournesol géant, maquette. Les dimensions du `catalog` sont à multiplier d'autant.
- La doc de chaque sorte (`schema`, enum `PropKind`) donne ses dimensions, son avant et comment
  la placer : modules à aligner tous les 2 m (`Fence`, `Railing`, `Cubicle`…), objets muraux
  posés en hauteur (`WallCabinet`, `Mirror`, `RangeHood`), objets suspendus au plafond
  (`CeilingLamp`, `AisleSign`). Un objet mural ou suspendu au-dessus d'un meuble ne compte pas
  comme un chevauchement.
- Tracés de terrain (`Markings`) : `FieldLine`, `FieldArc`, `FieldQuarterArc`, `FieldCircle`
  (longueur ou rayon par `height`) pour des limites sur mesure, ou un terrain complet aux cotes
  officielles (`SoccerPitch`, `BasketCourt`, `TennisCourt`…). Aplanir le relief dessous. Les
  marquages et objets posés sur une route (`Crosswalk`, `RoadDash`, voitures) se posent sur
  la chaussée.
- Chaque objet et chaque ouverture a des **parties** (`parts` dans `catalog`) : tissu et pieds
  d'un canapé, battant, cadre et poignée d'une porte… `paint` avec `"part": n` ne peint que la
  partie n ; sans `part`, c'est la partie 0, qui est aussi `color` / `texture` de l'élément.
  Exemple : `{"op": "paint", "target": {"Prop": 3}, "part": 1, "color": [0.1, 0.1, 0.1]}`.
- Pour repeindre plusieurs éléments d'un coup (les 4 murs d'une pièce…), lister les autres dans
  `targets` : `{"op": "paint", "target": {"Wall": 0}, "targets": [{"Wall": 1}, {"Wall": 2}], "texture": 12}`.
- Éclairage public : `StreetLamp` (4,5 m) ou `Lantern` (réverbère). `Lamp` et `CeilingLamp`
  éclairent l'intérieur.
- Pour une rangée régulière (haie, clôture, arbres, lampadaires), utiliser `place_row`.
- Pour changer un champ d'un objet déjà posé (hauteur d'un pilier, longueur d'une route,
  rotation…), utiliser `set`, qui garde les indices.

### Toits

`place_roof` couvre l'emprise des murs (`min` / `max` égaux à ceux de la `room`) et se pose au
`level` **au-dessus** des murs couverts : 1 pour une maison de plain-pied, 2 pour un étage. Le
faîtage suit le grand côté. Un toit plat est en zinc par défaut ; `texture` change la matière.
Formes : deux pans, quatre pans, plat, monopente, pyramide, mansarde, comble de grange
(`Gambrel`), `Saltbox`, papillon, sheds (dents de scie, toit d'usine ou de supermarché), voûte,
dôme, toit asiatique (`Pagoda`) et conique (tourelle).

Ouvertures : outre portes et fenêtres, des portes coulissantes (`SlidingDoor`, `Shoji`,
`PocketDoor`, `BarnDoor`, `AutoDoor`), battantes (`GlassDoor`, `DoubleDoor`), une porte de
garage basculante (`GarageDoor`) et une fenêtre coulissante, qui s'ouvrent toutes pendant la
visite. La baie vitrée, la vitre fixe et les fenêtres classiques restent fermées.

Pour un abri sur piliers : des `Pillar` de 3 m (un étage) et un toit `Flat` à `level` 1.

## Recettes

**Pièce simple** (`maison-simple.json`) : `room`, une porte, des fenêtres, des meubles collés aux
murs (`place_against` pour le canapé), puis `place_roof`.

**Plusieurs pièces** (`maison-deux-pieces.json`) :
- Une `room` pour l'enveloppe, puis des cloisons en `add_wall` (épaisseur 0,1, hauteur 3) dont
  les extrémités touchent les murs extérieurs.
- Ne pas accoler deux `room` : chacune aurait son propre mur sur la cloison commune.
- Mettre une porte dans chaque cloison qui sépare deux pièces qui communiquent.
- Nommer chaque pièce avec `name_room` (un point à l'intérieur suffit), ou avec le champ `name`
  de `room` pour une pièce sans cloison. `query monde.json room cuisine` la retrouve ensuite
  (sans tenir compte de la casse ni des accents). `unname_room` retire un nom ; pour renommer,
  `unname_room` puis `name_room`.

**Étage** (`maison-etage.json`) :
1. Poser les fenêtres du rez-de-chaussée **avant** `add_storey` : fenêtres, baies et
   portes-fenêtres sont recopiées à l'étage, les portes (`Door`) non.
2. Dans un monde à plusieurs bâtiments, donner `min` et `max` à `add_storey` pour ne monter que
   celui-là (un seul des deux est refusé).
3. `add_storey` copie le sol en entier. Le retirer et le remplacer par des `add_slab` qui
   laissent une trémie au-dessus de l'escalier.
4. Poser un `Stairs` (1 × 4,2 m pour 3 m) qui monte vers son avant. Son haut doit tomber sur
   un de ces sols : `check` signale un escalier sans trémie ou qui arrive dans le vide.
5. Le toit va à `level` 2.

**Rue** (`rue.json`) :
- La chaussée et les trottoirs sont des `add_road` parallèles.
- Le passage piéton est fait de bandes `add_road` étroites et claires posées en travers.
- Les lampadaires et les haies se posent avec `place_row`.
- Les voitures sont garées sur la chaussée, orientées dans le sens de la circulation.
- Une allée s'arrête au bord du trottoir.

**Extérieur** (`jardin.json`) :
- Le relief peut être généré avant ou après : tout mur ou sol posé au rez-de-chaussée aplanit
  le relief autour de lui.
- Piscine : coins en mètres entiers.
- Route : tronçons droits qui se suivent.
- Haie et clôture : `place_row` avec `step` 2 (modules jointifs, orientés le long de la rangée).

## Indices

- Chaque liste (murs, sols, toits, objets, piscines, routes) est numérotée dans l'ordre de
  création, et `apply` affiche les indices créés.
- `remove` décale de 1 les éléments suivants **de la même liste**. Dans un lot, supprimer du plus
  grand indice au plus petit. `set` modifie sans renuméroter.
- Une ouverture se désigne par `{"Opening": [mur, n]}`.

## Avertissements (`check`)

`check` signale, avec les indices en cause :
- objets qui se chevauchent ou traversent un mur ;
- objets qui flottent sans sol à leur étage ;
- escaliers sans trémie ou qui arrivent dans le vide ;
- portes d'étage qui donnent sur le vide ;
- toits posés au pied des murs ;
- murs en double ou enterrés ;
- piscines sous un sol ;
- routes qui traversent un mur ;
- noms de pièce posés dans un mur, ou deux noms dans la même pièce (une cloison manque ?).

Un monde prêt à rendre n'a **aucun avertissement**.

## Contrôle qualité avant de rendre un monde

- `check` ne signale rien.
- Chaque pièce a une porte, et la maison a une porte qui donne dehors.
- Les pièces de vie ont des fenêtres sur des murs extérieurs.
- Les meubles laissent un passage d'environ 0,8 m.
- Chaque étage habité a un sol, un escalier pour y monter, et un toit au-dessus du dernier étage.
- `shot` pour chaque étage (`--level`) et sous deux angles : comparer à la demande (proportions,
  orientation, nombre de pièces).
