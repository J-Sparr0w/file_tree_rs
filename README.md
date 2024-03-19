# Groot: File Tree in rust
Shows the contents of a directory in a tree structure.
* note: on windows tree command is already present.

## Features
- the command shows the no. of files and directories it printed
- it does not show hidden files and also does not count them while displaying the final file count output.
    * Note: it checks for files starting with a '.' in its name to determine whether it is hidden. On Windows, it also checks for the file attribute to determine whether it is hidden or not.
Usage: 
```tree [path]```


```[params]: optional parameters```


