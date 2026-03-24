export COREVM_SYSROOT="$HOME"/.corevm-dist/sysroot
# Prepend to PATH.
case ":$PATH:" in
*:"$HOME"/.corevm-dist/bin:*) ;;
*)
	export PATH="$HOME"/.corevm-dist/bin:"$PATH"
	;;
esac
