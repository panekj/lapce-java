xdg_data_dir :=  "$HOME/.local/share/"
plugin_dir := "plugins/oknozor.lapce-java/"

build:
    cargo make
    
install-stable: build
    mkdir -p {{xdg_data_dir}}/lapce-stable/{{plugin_dir}}
    yes | cp -i bin/lapce-java.wasm {{xdg_data_dir}}/lapce-stable/{{plugin_dir}}
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-stable/{{plugin_dir}}

install-debug: build
    mkdir -p {{xdg_data_dir}}/lapce-debug/{{plugin_dir}}    
    yes | cp -i bin/lapce-java.wasm {{xdg_data_dir}}/lapce-debug/{{plugin_dir}}
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-debug/{{plugin_dir}}
