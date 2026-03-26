export COREVM_HOME="${COREVM_HOME:-$HOME/.corevm}"
case ":$PATH:" in
*:"$COREVM_HOME"/bin:*) ;;
*) export PATH="$COREVM_HOME/bin${PATH:+:}$PATH" ;;
esac
