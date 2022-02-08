#!/bin/bash
# --- Initialization ---
set +x
path=$(pwd)
tput smcup # Switch to alternate screen so we preserve the terminal history
tput civis

trap clean_up_fail INT

clean_up() {
    tput rmcup
    tput cnorm
    echo "${1:-Goodbye! o/}"
    exit 0
}

clean_up_fail() {
    tput rmcup
    tput cnorm
    echo "${1:-Installation failed! :(}"
    exit 1
}

# --- Displaying ---

print_menu() {
    local function_arguments=($@)

    local selected_item="$1"
    local menu_items=(${function_arguments[@]:1})
    local menu_size="${#menu_items[@]}"

    for (( i = 0; i < $menu_size; ++i )) do
        if [ "$i" = "$selected_item" ]
        then echo -e "\033[2K\e[1m>\e[0m \e[1;33m${menu_items[i]}\e[0m"
        else echo -e "\033[2K  ${menu_items[i]}"
        fi
    done
}

run_menu() {
    local function_arguments=($@)

    local selected_item="$1"
    local menu_items=(${function_arguments[@]:1})
    local menu_size="${#menu_items[@]}"
    local menu_limit=$((menu_size - 1))

    clear
    print_menu "$selected_item" "${menu_items[@]}"
    
    while read -rsn1 input
    do
        case "$input" in
            $'\x1B')
                read -rsn1 -t 0.1 input
                if [ "$input" = "[" ] 
                then
                    read -rsn1 -t 0.1 input
                    case "$input"
                    in
                        A) # Arrow up
                            if [ "$selected_item" -ge 1 ]
                            then
                                selected_item=$((selected_item - 1))
                                clear
                                print_menu "$selected_item" "${menu_items[@]}"
                            fi;;
                        B) # Arrow down
                            if [ "$selected_item" -lt "$menu_limit" ]
                            then
                                selected_item=$((selected_item + 1))
                                clear
                                print_menu "$selected_item" "${menu_items[@]}"
                            fi;;
                    esac
                fi
                # stdin flush
                read -rsn5 -t 0.1;;
            "") # Enter
                return "$selected_item";;
        esac
    done
}

# --- Installation ---

check_installed() {
    if ! [ -x "$(command -v $1)" ]
    then clean_up_fail "Please check that you have $1 installed!"
    fi
}

install() {
    local selected_install_item=0
    local install_opts=("Download" "Compile" "Compile(Debug)" "Exit")
    run_menu "$selected_install_item" "${install_opts[@]}"
    local install_chosen="$?"

    case "$install_chosen" in
        0) echo "There is no release yet, please hold tight!";;
        1)
            echo "Testing dependencies..."
            check_installed git
            check_installed cargo
            echo "Setting up folders..."
            mkdir -p ~/.cache/
            rm -rf ~/.cache/vy/
            echo "Cloning repository..."
            cd ~/.cache/
            git clone https://github.com/azur1s/vy
            cd ~/.cache/vy/
            echo "Compiling..."
            cargo build --release
            mv ~/.cache/vy/target/release/vyc ~/bin/vyc
            clean_up "Done! Thanks a lot for trying out vy!";;
        2)
            echo "Testing dependencies..."
            check_installed git
            check_installed cargo
            echo "Setting up folders..."
            mkdir -p ~/.cache/
            rm -rf ~/.cache/vy/
            echo "Cloning repository..."
            cd ~/.cache/
            git clone https://github.com/azur1s/vy
            cd ~/.cache/vy/
            echo "Compiling..."
            cargo build
            mv ~/.cache/vy/target/debug/vyc ~/bin/vyc
            clean_up "Done! Thanks a lot for trying out vy!";;
        3) clean_up;;
    esac
}

uninstall() {
    echo "Uninstalling vyc..."
    rm ~/bin/vyc -f
    rm /usr/bin/vyc -f
    sleep 1s
    clean_up "Sad to see you go! Goodbye! o/"
}

selected_item=0
menu_opts=("Install" "Uninstall" "Exit")

run_menu "$selected_item" "${menu_opts[@]}"
menu_chosen="$?"

case "$menu_chosen" in
    0) install;;
    1) uninstall;;
    2) clean_up;;
esac
