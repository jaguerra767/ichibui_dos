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
    Admin = "Admin",
    Manager = "Manager",
    Operator = "Operator"
}

export enum DispenseType {
    Classic,
    LargeSmall
}


export enum IchibuState {
    Ready = "Ready",
    RunningClassic = "RunningClassic",
    RunningSized = "RunningSized",
    Cleaning = "Cleaning",
    Emptying = "Emptying",
}

export enum UiRequest {
    None = "None",
    SmallDispense = "SmallDispense",
    RegularDispense = "RegularDispense"
}