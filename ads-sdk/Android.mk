# Install ads-sdk.js, wrapper.js and ad-wrapper.html to $(TARGET_OUT)/kaios/http_root/sdk/ads/

LOCAL_PATH:= $(abspath $(call my-dir))
include $(CLEAR_VARS)
ADS_SDK_DIR := $(LOCAL_PATH)

LOCAL_MODULE := ads-sdk
LOCAL_MODULE_CLASS := ETC
LOCAL_MODULE_TAGS := optional
LOCAL_MODULE_PATH := $(TARGET_OUT)/kaios/http_root/sdk/ads/
include $(BUILD_PREBUILT)

$(LOCAL_BUILT_MODULE):
	@echo "adk-sdk: install from $(ADS_SDK_DIR)"

$(LOCAL_INSTALLED_MODULE):
	@mkdir -p $(@D)
	@cp $(ADS_SDK_DIR)/ads-sdk.js $(@D)
	@cp $(ADS_SDK_DIR)/wrapper.js $(@D)
	@cp $(ADS_SDK_DIR)/ad-wrapper.html $(@D)
