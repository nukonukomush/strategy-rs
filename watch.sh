if [ $1 = "rs" ]; then
    cargo watch -w src -s 'cargo test || say -v Daniel test failed'
elif [ $1 = "py" ]; then
    ptw --ext .rs,.py --beforerun 'cargo build' --onfail 'say -v Daniel test failed'
fi
