# TODO — voxelith

## Prochains candidats (à arbitrer)

- [ ] Hotbar visuelle (cases + bloc sélectionné en bas d'écran)
- [ ] Caves & overhangs : density function 3D dans le worldgen
- [ ] Arbres / structures (feature placement par biome)
- [ ] Eau transparente (pass de rendu trié, blend)
- [ ] Inventaire / système de santé (survival complet)
- [ ] Settings fonctionnels (slider render distance, FOV, sensibilité)
- [ ] GPU-driven `multi_draw_indirect` + occlusion Hi-Z
- [ ] LOD par bandes de distance (render distance ≫)

## Backlog

- [ ] Suppression de monde dans WorldSelect
- [ ] Mipmaps sur le texture array
- [ ] Bit-packing réel des indices de palette
- [ ] Sauvegarde des chunks générés (pas seulement les overrides)

## Fait

- [x] Streaming infini async (rayon + channel)
- [x] Worldgen multi-noise + biomes + greedy meshing + textures procédurales
- [x] Frustum culling CPU, fog de distance
- [x] WorldStore éditable (overrides) + collision + raycast
- [x] Player physics + gamemodes creative/survival
- [x] Édition casser/poser + palette + re-mesh
- [x] UI maison (police bitmap, overlay 2D, kit immediate-mode)
- [x] HUD (crosshair, infos) + chat + commandes
- [x] Menus (main, world select, create, settings, pause)
- [x] Persistance multi-mondes JSON + sauvegarder & quitter
