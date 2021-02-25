#include "strings.h"

typedef enum {
    JSON_ELEMENT_STRING,
    JSON_ELEMENT_ARRAY_OF_OBJECTS,
    JSON_ELEMENT_ARRAY_OF_STRINGS,
    JSON_ELEMENT_ARRAY_OF_ARRAYS,
    JSON_ELEMENT_ARRAY_EMPTY,
    JSON_ELEMENT_OBJECT
} json_element_type;


typedef struct s_json_element{
    json_element_type type;
    string_view name;
    void* content;
    struct s_json_element* next;
} json_element;

json_element* new_json_element(){
    json_element* elem = malloc(sizeof(json_element));
    elem->content = (void*)0;
}

json_element* json_element_object_get_child_with_name(json_element* parent, const char* name){
    json_element* elem = parent->content;
    while(elem != (void*)0){
        if(string_view_equals_c_str(&elem->name, name)) return elem;
        elem = elem->next;
    }
    return (void*)0;
}

/**
 * ! Does ot work with end Nodes (e.g. String) as parent
 */
void json_element_add_as_child(json_element* parent, void* child){
    if(parent->content == (void*)0){
        parent->content = child;
    } else if(parent->type != JSON_ELEMENT_STRING){
        json_element* current = (json_element*)parent->content;
        while(current->next != (void*)0){
            current = current->next;
        }
        current->next = child;
    } 
} 

const char* JSON_SYNTAX_ALLOWED_SKIPPABLES = " \t\n";

typedef enum {
    STATUS_OK = 0,
    ERROR_UNEXPECTED_END_OF_SOURCE = -100,
    ERROR_UNEXPECTED_TOKEN = -200,
    ERROR_ARRAY_OF_MIXED_TYPES = -300
} json_parser_errcodes;
#define END_OF_FILE_CHECK(character) if(character == '\0') return ERROR_UNEXPECTED_END_OF_SOURCE;
#define RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index) if(index < 0) { return index; }
#define ERROR_IF_NOT_SKIPPABLE_CHAR(character)  if(!char_in_char_set(character, JSON_SYNTAX_ALLOWED_SKIPPABLES)){ return ERROR_UNEXPECTED_TOKEN; }

typedef struct {
    json_element* root;
    bool success;
    json_parser_errcodes code;
} json_parser_result;

int json_skip_til_char_only_skippables(const char* source, int index, char expected){
    char c = source[index];
    while(c != expected) {
        index++;
        ERROR_IF_NOT_SKIPPABLE_CHAR(c);
        c = source[index];
        END_OF_FILE_CHECK(c);
    };

    return index;
}

int json_jump_to_next_unskippable_char(const char* source, int index){
    char c = '\t';
    while(char_in_char_set(c, JSON_SYNTAX_ALLOWED_SKIPPABLES)){
        c = source[index];
        index++;
        END_OF_FILE_CHECK(c);
    }

    return index - 1;
}

int json_parse_string_til_next_quotation_mark(const char* source, int index, string_view* dest){
    dest->src = source;
    dest->begin = index;
    
    char c = source[index];
    while (c != '"'){
        index++;
        c = source[index];
        END_OF_FILE_CHECK(c);
    }

    dest->end = index;
    return index + 1;
}

int json_parse_object(const char* source, int index, json_element* parent);


#define SET_ARRAY_ELEMENT_TYPE_IF_EMPTY(elem, _type) if(elem->type == JSON_ELEMENT_ARRAY_EMPTY) elem->type = _type;
int json_parse_array(const char* source, int index, json_element* parent){
    
    bool element_expected = true;
    bool empty_array = true;

    char c;
    do {
        index = json_jump_to_next_unskippable_char(source, index);
        c = source[index];
        switch(c){
            case '{': {
                if(!element_expected) return ERROR_UNEXPECTED_TOKEN;
                SET_ARRAY_ELEMENT_TYPE_IF_EMPTY(parent, JSON_ELEMENT_ARRAY_OF_OBJECTS);
                if(parent->type != JSON_ELEMENT_ARRAY_OF_OBJECTS) return ERROR_ARRAY_OF_MIXED_TYPES;
                
                json_element* object = new_json_element();
                string_view_clear_to_empty(&object->name);
                object->type = JSON_ELEMENT_OBJECT;

                index = json_parse_object(source, index+1, object);
                RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
                json_element_add_as_child(parent, object);
                empty_array = false;
                element_expected = false;
                break;
            }

            case ',': {
                if(element_expected) return ERROR_UNEXPECTED_TOKEN;
                element_expected = true;
                index++;
                break;
            }

            case '"': {
                if(!element_expected) return ERROR_UNEXPECTED_TOKEN;
                SET_ARRAY_ELEMENT_TYPE_IF_EMPTY(parent, JSON_ELEMENT_ARRAY_OF_STRINGS);
                if(parent->type != JSON_ELEMENT_ARRAY_OF_STRINGS) return ERROR_ARRAY_OF_MIXED_TYPES;

                json_element* elem = new_json_element();
                string_view_clear_to_empty(&elem->name);
                elem->type = JSON_ELEMENT_STRING;
                elem->content = malloc(sizeof(string_view));

                index = json_parse_string_til_next_quotation_mark(source, index + 1, elem->content);
                RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);

                json_element_add_as_child(parent, elem);

                empty_array = false; 
                element_expected = false;
                break;
            }

            case ']': {
                if(element_expected && !empty_array) return ERROR_UNEXPECTED_TOKEN;
                break;
            }
        }        
    } while(c != ']');
    
    return index + 1;

}

int json_parse_object_element(const char* source, int index, json_element* parent){
    /**
     * Create child element
     * ? Name
     * ? Content
     */
    json_element* elem = (json_element*)new_json_element();
    /**
     * Parse name of element
     * ? Content
     */
    index = json_parse_string_til_next_quotation_mark(source, index, &elem->name);
    RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
    /**
     * Skip til ':' occures and content follows
     */
    index = json_skip_til_char_only_skippables(source, index, ':');
    RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
    index++; // Skip ':'
    /**
     * Goto next unskippable char
     * ! '"' => STRING
     * ! '{' => OBJECT
     * ! '[' => ARRAY
     */
    index = json_jump_to_next_unskippable_char(source, index);
    RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);

    char key_char = source[index];
    switch(key_char){
        //! STRING
        case '"': {
            // Set element type to String
            elem->type = JSON_ELEMENT_STRING;
            /**
             * Create new string_view container and parse content of string into it
             */
            string_view* content = (string_view*)malloc(sizeof(string_view));
            index = json_parse_string_til_next_quotation_mark(source, index + 1, content);
            RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
            elem->content = content;
            break;
        }
        //! OBJECT
        case '{': {
            // Set element type to Object
            elem->type = JSON_ELEMENT_OBJECT;
            /**
             * Parse content recursively => This element becomes parent
             */
            index = json_parse_object(source, index + 1, elem);
            RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
            break;
        }
        //! ARRAY
        case '[': {
            // Set element type to (yet) empty array
            elem->type = JSON_ELEMENT_ARRAY_EMPTY;
            /**
             * Parese content recursively => This element becomes parent
             */
            index = json_parse_array(source, index + 1, elem);
            RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
            break;
        }
    }

    /**
     * Append current element to parent element
     */
    json_element_add_as_child(parent, elem);
    return index;
}



int json_parse_object(const char* source, int index, json_element* parent){

    bool element_expected = true;
    bool empty_object = true;

    while(1){
        index = json_jump_to_next_unskippable_char(source, index);
        RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
        char c = source[index];
        if (c == '"'){
            if(!element_expected) return ERROR_UNEXPECTED_TOKEN;
            index = json_parse_object_element(source, index+1, parent);
            RETURN_ERROR_CODE_IF_ERRORS_OCCURED(index);
            element_expected = false;
            empty_object = false;
        } else if (c == ','){
            if(element_expected)
                return ERROR_UNEXPECTED_TOKEN;
            element_expected = true;
            index++;
        } else if (c == '}'){
            if(!empty_object && element_expected)
                return ERROR_UNEXPECTED_TOKEN;
            else
                break;
        } else {
            return ERROR_UNEXPECTED_TOKEN;
        }
    }

    return index+1;
}

//int json_parse_element_name(const char* source, int index, json_element* parent){}


json_parser_result json_parse_source(const char* source, const char* with_name){

    json_parser_result result;
    result.root = new_json_element();;

    string_view name = (string_view){with_name, 0, string_len(with_name)};

    result.root->name = name;
    result.root->type = JSON_ELEMENT_OBJECT;

    int index = 0;
    index = json_jump_to_next_unskippable_char(source, index);
    if(index >= 0){
        char c = source[index];
        if(c == '{'){
            index = json_parse_object(source, index+1, result.root);
        } else {
            index = ERROR_UNEXPECTED_TOKEN;
        }
    }

    if(index < 0){
        result.success = false;
        result.code = index;
    } else {
        result.success = true;
        result.code = STATUS_OK;
    }
    

    return result;
}

