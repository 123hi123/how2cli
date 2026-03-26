# how2cli shell integration for zsh
# Add to ~/.zshrc: source /path/to/how2cli/shell/h.zsh
#
# Uses alias (not function) because zsh expands globs BEFORE
# calling functions, so noglob must be applied via alias.

alias h='noglob command h'
alias ht='noglob command ht'
