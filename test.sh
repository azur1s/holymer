in=$1             # Get first's file name 
noext=${in%.*}    # Remove extension
name=${noext##*/} # Remove path

echo $2
# if $2 equal to "noecho"
if [ $2 = "noecho" ];
then
    make debug; echo ""; vyc compile $noext.vy; echo ""
    cat $noext.vy; echo -e "\n"; cat $name.vyir; echo ""
    vyc run $name.vyir
else
    make debug
    vyc compile $noext.vy
    echo -e   "------------------------------------------- SOURCE"
    cat $noext.vy
    echo -e "\n----------------------------------------- COMPILED"
    cat $name.vyir
    echo -e   "------------------------------------------- OUTPUT"
    blspc run $name.vyir
    echo -e   "--------------------------------------------------"
fi
