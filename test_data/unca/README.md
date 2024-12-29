Tests are taken from UNCA's public ECE109 [course materials](https://www.cs.unca.edu/~brock/classes/Spring2009/ece109/Lectures/Examples).
`examples.html` retrieved December 29, 2024.
The author is assumed to be J Dean Brock.
The work is presumed to be provided for open source due to its distribution method.

`split_apart` contains each program used for testing.
`assembled` contains the programs as assembled by PennSim.
It was created with the following script (run in this directory).
```bash
cp -r split_apart assembled
for f in assembled/*; do
    java -jar ../../penn_sim/PennSim.jar -t -s <(printf '%s\n%s\n' "as $f" 'quit');
done
rm assembled/*.asm
```
