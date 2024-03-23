PROJECT_NAME := revere
BINARY_PATH := /usr/local/bin
SERVICE_PATH := $(HOME)/.config/systemd/user
CONFIG_DIR := $(HOME)/.config/revere
CONFIG_FILE := $(CONFIG_DIR)/config.toml
USER_ID := $(shell id -u)

# Default target
all: build install config setup-service start-service

# Build revere
build:
	cargo build --release

# Install revere
install: build
	sudo cp target/release/${PROJECT_NAME} ${BINARY_PATH}

# Configure revere
config:
	@echo "Creating config file in /.config/revere"
	@if [ ! -d "$(CONFIG_DIR)" ]; then \
	    mkdir -p $(CONFIG_DIR); \
	fi
	@if [ ! -f "$(CONFIG_FILE)" ]; then \
	    cp config.template.toml $(CONFIG_FILE); \
	else \
	    echo "Configuration file already exists."; \
	fi

# Setup revere as a daemon with systemd
setup-service:
	@echo "Setting up Revere daemon..."
	@# Create the systemmd service file
	@sudo sh -c 'echo "[Unit]" > $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "Description=$(PROJECT_NAME) service" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "[Service]" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "Type=simple" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "ExecStart=$(BINARY_PATH)/$(PROJECT_NAME)" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "Restart=on-failure" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "Environment=HOME=$(HOME)" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(USER_ID)/bus" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "[Install]" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'
	@sudo sh -c 'echo "WantedBy=multi-user.target" >> $(SERVICE_PATH)/$(PROJECT_NAME).service'

# Enable and start the service
start-service:
	systemctl --user enable $(PROJECT_NAME)
	systemctl --user start $(PROJECT_NAME)

# Stop and disable the service
stop-service:
	systemctl --user stop $(PROJECT_NAME)
	systemctl --user disable $(PROJECT_NAME)

# Uninstall the binary and service
uninstall:
	sudo rm -f $(BINARY_PATH)/$(PROJECT_NAME)
	sudo rm -f $(SERVICE_PATH)/$(PROJECT_NAME).service

.PHONY: build install setup-service start-service stop-service uninstall
