# Moleco

Moleco stands for **mole**cule to **co**lor. It generates unique color swatch for given substance based on its InChI notation. It can also generate color identification for mixture using MInChI notation.

## How to run

`moleco generate "InChI=1S/C8H10N4O2/c1-10-4-9-6-5(10)7(13)12(3)8(14)11(6)2/h4H,1-3H3" --print`

That will generate a color swatch for caffeine.

![Caffeine swatch](readme/caffeine.png)

## Installation

TODO

## Support for mixtures

Of course in nature there is much more likely to see mixtures instead of single substances, so MInChI is supported as well. You can generate toothpaste:

```
moleco generate "MInChI=0.00.1S/C12H26O4S.Na/c1-2-3-4-5-6-7-8-9-10-11-12-16-17(13,14)15;/h2-12H2,1H3,(H,13,14,15);/q;+1/p-1&C3H8O3/c4-1-3(6)2-5/h3-6H,1-2H2&C7H5NO3S.Na/c9-7-5-3-1-2-4-6(5)12(10,11)8-7;/h1-4H,(H,8,9);/q;+1/p-1&Ca.H3O4P.2H2O/c;1-5(2,3)4;;/h;(H3,1,2,3,4);2*1H2/q+2;;;/p-2&FH2O3P.2Na/c1-5(2,3)4;;/h(H2,2,3,4);;/q;2*+1/p-2&H2O/h1H2/n{6&2&&5&3&4&1}/g{215wf-3&25wf-2&1wf-2&8wf-3&2wf-3&5wf-1&15wf-3}" --print
```

![Toothpaste](readme/toothpaste.png)

## Motivation

TODO

## How mixture bar sizes are calculated

TODO

## Questions

### Are there collisions?

TODO

### Why no support for InChIKey?

TODO

### Why the shape?

TODO

## References

### IhChI and MInChI

  * https://jcheminf.biomedcentral.com/articles/10.1186/s13321-015-0068-4
  * https://jcheminf.biomedcentral.com/articles/10.1186/s13321-019-0357-4
  * http://molmatinf.com/minchidemo/
  * https://github.com/IUPAC/MInChI_demo

### Color spaces

  * https://paletton.com/
  * https://ericportis.com/posts/2024/okay-color-spaces/

### PubChem resources
  * https://pubchem.ncbi.nlm.nih.gov/
  * https://pubchem.ncbi.nlm.nih.gov/docs/compound-vs-substance

