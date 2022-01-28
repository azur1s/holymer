in=$1             # Get first's file name 
noext=${in%.*}    # Remove extension
name=${noext##*/} # Remove path

make debug
blspc compile $noext.blsp
echo -e   "------------------------------------------- SOURCE"
cat $noext.blsp
echo -e "\n----------------------------------------- COMPILED"
cat $name.bsm
echo -e   "------------------------------------------- OUTPUT"
blspc run $name.bsm
echo -e   "--------------------------------------------------"
