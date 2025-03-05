import React from "react";
import { DispenseType, Ingredient} from "./types";
import ClassicModeDispense from "./classic-mode-dispense";
import SizedModeDispense from "./sized-mode-dispense";


interface DispenseScreenProps{
    snack: Ingredient | undefined,
    mode: DispenseType
}

const DispenseScreen: React.FC<DispenseScreenProps> = ({snack, mode}) => {
    const classicModeOn = mode == DispenseType.Classic;
    const smallLargeModeOn = mode === DispenseType.LargeSmall;
    return (
        <div>
            {classicModeOn && <ClassicModeDispense snack={snack}></ClassicModeDispense>}
            {smallLargeModeOn && <SizedModeDispense snack={snack}></SizedModeDispense>}
        </div>
    )
}

export default DispenseScreen;