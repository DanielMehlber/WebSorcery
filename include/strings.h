#include "stdlib.h"


#ifndef bool
#define bool _Bool
#define true 1
#define false 0
#endif

int string_len(const char* string){
    char c = string[0];
    int index = 0;
    while(c != '\0'){
        index++;
        c = string[index];
    }
    return index;
}

typedef struct {
    const char* src;
    int begin;
    int end;
} string_view;

char string_view_get_char_at_index(string_view* view, int index){
    return view->src[view->begin + index];
}

bool string_view_equals(string_view* one, string_view* two){
    int length = one->end - one->begin;
    if(length != (two->end - two->begin)) return false;
    for(int i = 0; i < length; i++){
        if(string_view_get_char_at_index(one, i) != string_view_get_char_at_index(two, i))
            return false;
    }
    return true;
}

bool string_view_equals_c_str(string_view* one, const char* two){
    int length = one->end - one->begin;
    if(length != string_len(two)) return false;
    for(int i = 0; i < length; i++){
        if(string_view_get_char_at_index(one, i) != two[i])
            return false;
    }
    return true;
}

int string_view_get_next_index_of(string_view* string, char c, int index){
    int length = string->end - string->begin;
    for(; index < length; index++){
        char token = string_view_get_char_at_index(string, index);
        if(token == c) return index;
    }
    return -1;
}

struct s_string_builder_node {
    int length;
    const char* content;
    struct s_string_builder_node* next;
};

struct s_string_builder_node* new_string_builder_node(const char* text){
    struct s_string_builder_node* node = malloc(sizeof(struct s_string_builder_node));
    node->content = text;
    node->length = string_len(text);
    node->next = (void*)0; 
    return node;
}

typedef struct string_builder{
    int length;
    struct s_string_builder_node* first;
    struct s_string_builder_node* last;
} string_builder;

string_builder* new_string_builder(){
    string_builder* builder = malloc(sizeof(string_builder));
    //builder->first = (void*)0;
    //builder->last = (void*)0;
    builder->length = 0;
    return builder;
}

void string_builder_add_string(string_builder* builder, const char* text){
    struct s_string_builder_node* new_node = new_string_builder_node(text);
    if(builder->last != (void*)0){
        builder->last->next = new_node;
    } else {
        builder->first = new_node;
    }

    builder->last = new_node;

    builder->length += new_node->length;
}

const char* string_builder_to_c_string(string_builder* builder){
    char* string = malloc(sizeof(char) * (builder->length + 1));
    struct s_string_builder_node* current = builder->first;
    int index = 0;
    while(current != (void*)0){
        for(int i = 0; i < current->length; i++){
            string[index] = current->content[i];
            index++;
        }
        current = current->next;
    }

    string[builder->length + 1] = '\0';
    return string;
}


const char* string_view_to_c_string(string_view* string){
    int length = string->end - string->begin;

    char* cstr = (char*) malloc(sizeof(char) * (length+1));

    for(int i = 0; i < length; i++){
        cstr[i] = string_view_get_char_at_index(string, i);
    }

    cstr[length] = '\0';
    return cstr;
}


void string_view_clear_to_empty(string_view* view){
    view->begin = 0;
    view->end = 0;
    view->src = (void*)0;
}

bool char_in_char_set(char c, const char* set){
    int index = 0;
    char c_set = set[index];
    while(c_set != '\0'){
        c_set = set[index];
        if( c == c_set)
            return true;
        index++;
    }
    return false;
}