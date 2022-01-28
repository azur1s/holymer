in=$1             # Get first's file name 
noext=${in%.*}    # Remove extension
name=${noext##*/} # Remove path

echo $2
# if $2 equal to "noecho"
if [ $2 = "noecho" ];
then
    make debug; echo ""; blspc compile $noext.blsp; echo ""
    cat $noext.blsp; echo -e "\n"; cat $name.bsm; echo ""
    blspc run $name.bsm
else
    make debug
    blspc compile $noext.blsp
    echo -e   "------------------------------------------- SOURCE"
    cat $noext.blsp
    echo -e "\n----------------------------------------- COMPILED"
    cat $name.bsm
    echo -e   "------------------------------------------- OUTPUT"
    blspc run $name.bsm
    echo -e   "--------------------------------------------------"
fi
