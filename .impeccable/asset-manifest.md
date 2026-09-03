# Asset production manifest

## Produce

None. The approved comp contains no photographic, illustrated, or textured region that requires raster production.

## Direct

None.

## Semantic

| id               | implementation                                                                                                                             | notes                                                                             | qa_status |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------- | --------- |
| hero-statement   | Semantic `h1` with separate product-name and descriptor spans; Archivo Variable supplies the width and weight; CSS owns scale and wrapping | Text remains selectable and responsive                                            | accepted  |
| signal-bleed     | Layered CSS radial gradients behind the hero                                                                                               | This is a geometric color field in the approved comp, not an image-native texture | accepted  |
| primary-action   | Semantic external link with Tailwind layout, color, focus, and state utilities                                                             | One primary action only                                                           | accepted  |
| terminal-preview | Semantic `figure`, `figcaption`, and typed terminal-line data rendered through nested Svelte `{#each}` blocks                              | CSS owns pane shape, rule, depth, and responsive sizing                           | accepted  |
| lifecycle-rail   | Semantic ordered list with a CSS-owned connector and signal animation                                                                      | Five stages become vertical rows on compact screens                               | accepted  |

## Execution order

1. Hero statement and signal field.
2. Primary action.
3. Terminal pane.
4. Lifecycle rail and responsive adaptation.

## Blockers

None.

## Assumptions

- The approved comp's broad green field is intentionally clean and code-native; it does not depict physical texture, dimensional machinery, or illustration.
- Window markers are interface geometry and remain semantic CSS rather than icons or raster assets.
