export interface UiData {
    id: number
    label: string,
    img: string,
    serving_size: number
    ingredients: string
}

export interface Ingredient {
    id: number
    name: string
    img_filename: string
    base64_img: string 
}


export enum User {
    None = "None",
    Admin = "ADMIN",
    Manager = "MANAGER",
    Operator = "OPERATOR"
}

export enum DispenseType {
    Classic,
    LargeSmall
}


export enum RunState {
    Ready,
    Running,
    Cleaning,
    Emptying,
}