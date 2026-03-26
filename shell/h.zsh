# how2cli shell integration for zsh
# Add to ~/.zshrc: source /path/to/how2cli/shell/h.zsh
#
# This wrapper prevents zsh from treating ?, *, [] etc. as glob patterns
# so you can type: h is this hard? without quoting

h() {
    noglob command h "$@"
}

ht() {
    noglob command ht "$@"
}
