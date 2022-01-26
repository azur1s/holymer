#!/bin/bash
tput sc

function clear_print() {
    tput rc
}

function print_menu() {
	local function_arguments=($@)

	local selected_item="$1"
	local menu_items=(${function_arguments[@]:1})
	local menu_size="${#menu_items[@]}"

	for (( i = 0; i < $menu_size; ++i ))
	do
		if [ "$i" = "$selected_item" ]
		then
			echo "* ${menu_items[i]}"
		else
			echo "  ${menu_items[i]}"
		fi
	done
}

function run_menu() {
	local function_arguments=($@)

	local selected_item="$1"
	local menu_items=(${function_arguments[@]:1})
	local menu_size="${#menu_items[@]}"
	local menu_limit=$((menu_size - 1))

	clear_print
	print_menu "$selected_item" "${menu_items[@]}"
	
	while read -rsn1 input
	do
		case "$input"
		in
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
								clear_print
								print_menu "$selected_item" "${menu_items[@]}"
							fi
							;;
						B) # Arrow down
							if [ "$selected_item" -lt "$menu_limit" ]
							then
								selected_item=$((selected_item + 1))
								clear_print
								print_menu "$selected_item" "${menu_items[@]}"
							fi
							;;
					esac
				fi
				read -rsn5 -t 0.1 # stdin flush
				;;
			"") # Enter
				return "$selected_item"
				;;
		esac
	done
}

selected_item=0
menu_items=("Install" "Uninstall" "Exit")

run_menu "$selected_item" "${menu_items[@]}"
menu_result="$?"

case "$menu_result"
in
	0)
		echo "Install selected"
		;;
	1)
		echo "Uninstall selected"
		;;
	2)
		echo "Goodbye"
		;;
esac