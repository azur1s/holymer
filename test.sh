in=$1         # Get first's file name 
name=${in%.*} # Remove extension

make debug
blspc compile $name.blsp
echo -e   "------------------------------------------- SOURCE"
cat ./$name.blsp
echo -e "\n----------------------------------------- COMPILED"
cat ./$name.bsm
echo -e   "------------------------------------------- OUTPUT"
blspc run $name.bsm
echo -e   "--------------------------------------------------"
