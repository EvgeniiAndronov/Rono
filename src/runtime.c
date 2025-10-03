#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <curl/curl.h>

// Runtime function for console output
void rono_print_int(int64_t value) {
    printf("%lld\n", (long long)value);
}

void rono_print_float(double value) {
    printf("%f\n", value);
}

void rono_print_bool(int8_t value) {
    printf("%s\n", value ? "true" : "false");
}

void rono_print_string(const char* str) {
    if (str) {
        printf("%s\n", str);
    } else {
        printf("(null)\n");
    }
}

// String interpolation support
void rono_print_interpolated(const char* format, int64_t value) {
    // Simple implementation: replace {} with %lld
    char* result = malloc(strlen(format) + 32); // Extra space for number
    char* src = (char*)format;
    char* dst = result;
    
    while (*src) {
        if (*src == '{' && *(src + 1) == '}') {
            // Replace {} with the value
            dst += sprintf(dst, "%lld", (long long)value);
            src += 2; // Skip {}
        } else {
            *dst++ = *src++;
        }
    }
    *dst = '\0';
    
    printf("%s\n", result);
    free(result);
}

// Formatted output with interpolation
void rono_print_format_int(const char* format, int64_t value) {
    if (format == NULL) {
        // Default format for when we can't pass string constants yet
        printf("%lld\n", (long long)value);
    } else {
        rono_print_interpolated(format, value);
    }
}

// Console input functions
char* rono_input_string() {
    char* buffer = malloc(1024); // Allocate buffer for input
    if (buffer == NULL) {
        return NULL;
    }
    
    if (fgets(buffer, 1024, stdin) != NULL) {
        // Remove trailing newline if present
        size_t len = strlen(buffer);
        if (len > 0 && buffer[len - 1] == '\n') {
            buffer[len - 1] = '\0';
        }
        return buffer;
    } else {
        free(buffer);
        return NULL;
    }
}

int64_t rono_input_int() {
    char* input = rono_input_string();
    if (input == NULL) {
        return 0;
    }
    
    int64_t result = strtoll(input, NULL, 10);
    free(input);
    return result;
}

double rono_input_float() {
    char* input = rono_input_string();
    if (input == NULL) {
        return 0.0;
    }
    
    double result = strtod(input, NULL);
    free(input);
    return result;
}

int8_t rono_input_bool() {
    char* input = rono_input_string();
    if (input == NULL) {
        return 0;
    }
    
    int8_t result = 0;
    if (strcmp(input, "true") == 0 || strcmp(input, "1") == 0) {
        result = 1;
    } else if (strcmp(input, "false") == 0 || strcmp(input, "0") == 0) {
        result = 0;
    }
    
    free(input);
    return result;
}

// Random number generation functions
static int rono_rand_initialized = 0;

void rono_rand_init() {
    if (!rono_rand_initialized) {
        srand((unsigned int)time(NULL));
        rono_rand_initialized = 1;
    }
}

int64_t rono_rand_int(int64_t min, int64_t max) {
    rono_rand_init();
    
    if (min > max) {
        // Swap if min > max
        int64_t temp = min;
        min = max;
        max = temp;
    }
    
    if (min == max) {
        return min;
    }
    
    // Generate random number in range [min, max]
    int64_t range = max - min + 1;
    return min + (rand() % range);
}

double rono_rand_float(double min, double max) {
    rono_rand_init();
    
    if (min > max) {
        // Swap if min > max
        double temp = min;
        min = max;
        max = temp;
    }
    
    if (min == max) {
        return min;
    }
    
    // Generate random float in range [min, max)
    double range = max - min;
    return min + ((double)rand() / RAND_MAX) * range;
}

char* rono_rand_string(int64_t length) {
    rono_rand_init();
    
    if (length <= 0) {
        char* empty = malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }
    
    char* result = malloc(length + 1);
    if (!result) {
        return NULL;
    }
    
    const char charset[] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const int charset_size = sizeof(charset) - 1;
    
    for (int64_t i = 0; i < length; i++) {
        result[i] = charset[rand() % charset_size];
    }
    result[length] = '\0';
    
    return result;
}

// Generate random character in range (simplified implementation)
char* rono_rand_char_range(const char* from, const char* to) {
    rono_rand_init();
    
    if (!from || !to || strlen(from) == 0 || strlen(to) == 0) {
        char* result = malloc(2);
        if (result) {
            result[0] = 'a';
            result[1] = '\0';
        }
        return result;
    }
    
    char from_char = from[0];
    char to_char = to[0];
    
    if (from_char > to_char) {
        // Swap if from > to
        char temp = from_char;
        from_char = to_char;
        to_char = temp;
    }
    
    char* result = malloc(2);
    if (!result) {
        return NULL;
    }
    
    // Generate random character in range [from_char, to_char]
    int range = to_char - from_char + 1;
    result[0] = from_char + (rand() % range);
    result[1] = '\0';
    
    return result;
}

// HTTP response structure
typedef struct {
    char* data;
    size_t size;
} HttpResponse;

// Callback function for writing HTTP response data
static size_t WriteCallback(void* contents, size_t size, size_t nmemb, HttpResponse* response) {
    size_t realsize = size * nmemb;
    char* ptr = realloc(response->data, response->size + realsize + 1);
    
    if (ptr == NULL) {
        // Out of memory
        return 0;
    }
    
    response->data = ptr;
    memcpy(&(response->data[response->size]), contents, realsize);
    response->size += realsize;
    response->data[response->size] = 0; // Null terminate
    
    return realsize;
}

// Initialize curl (called once)
static int curl_initialized = 0;

void rono_http_init() {
    if (!curl_initialized) {
        curl_global_init(CURL_GLOBAL_DEFAULT);
        curl_initialized = 1;
    }
}

// HTTP GET function
char* rono_http_get(const char* url) {
    rono_http_init();
    
    CURL* curl;
    CURLcode res;
    HttpResponse response = {0};
    
    curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, url);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
        curl_easy_setopt(curl, CURLOPT_USERAGENT, "Rono-HTTP/1.0");
        curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L); // 30 second timeout
        
        res = curl_easy_perform(curl);
        curl_easy_cleanup(curl);
        
        if (res != CURLE_OK) {
            if (response.data) {
                free(response.data);
            }
            return NULL;
        }
    }
    
    return response.data; // Caller must free this
}

// HTTP POST function
char* rono_http_post(const char* url, const char* data) {
    rono_http_init();
    
    CURL* curl;
    CURLcode res;
    HttpResponse response = {0};
    
    curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, url);
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, data);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
        curl_easy_setopt(curl, CURLOPT_USERAGENT, "Rono-HTTP/1.0");
        curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);
        
        res = curl_easy_perform(curl);
        curl_easy_cleanup(curl);
        
        if (res != CURLE_OK) {
            if (response.data) {
                free(response.data);
            }
            return NULL;
        }
    }
    
    return response.data; // Caller must free this
}

// HTTP PUT function
char* rono_http_put(const char* url, const char* data) {
    rono_http_init();
    
    CURL* curl;
    CURLcode res;
    HttpResponse response = {0};
    
    curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, url);
        curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "PUT");
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, data);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
        curl_easy_setopt(curl, CURLOPT_USERAGENT, "Rono-HTTP/1.0");
        curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);
        
        res = curl_easy_perform(curl);
        curl_easy_cleanup(curl);
        
        if (res != CURLE_OK) {
            if (response.data) {
                free(response.data);
            }
            return NULL;
        }
    }
    
    return response.data; // Caller must free this
}

// HTTP DELETE function
char* rono_http_delete(const char* url) {
    rono_http_init();
    
    CURL* curl;
    CURLcode res;
    HttpResponse response = {0};
    
    curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, url);
        curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "DELETE");
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
        curl_easy_setopt(curl, CURLOPT_USERAGENT, "Rono-HTTP/1.0");
        curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);
        
        res = curl_easy_perform(curl);
        curl_easy_cleanup(curl);
        
        if (res != CURLE_OK) {
            if (response.data) {
                free(response.data);
            }
            return NULL;
        }
    }
    
    return response.data; // Caller must free this
}