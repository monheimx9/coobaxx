#!/bin/bash

session="coobaxx"

tmux new-session -d -s $session

tmux rename-window -t 0 "Nvim"
tmux send-keys -t "Nvim" "nvim src/main.rs" C-m

tmux new-window -t $session:1 -n "bacon runner"
tmux send-keys -t "bacon runner" "bacon run -- --release" C-m
tmux send-keys -t "bacon runner" F5

tmux attach-session -t $session:0
