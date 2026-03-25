corevm_home="${COREVM_HOME:-$HOME/.corevm}"
case ":$PATH:" in
*:"$corevm_home"/bin:*) ;;
*)
	export PATH="$corevm_home/bin${PATH:+:}$PATH"
	;;
esac
unset corevm_home
