import React, { useEffect, useState } from "react";
import { DispenseType, Ingredient, UiRequest} from "./types";
import recycle from "./assets/recycle.svg"
import { invoke } from "@tauri-apps/api/core";
import SvgViewer from "./components/svg-viewer";
import { Button } from "./components/ui/button";
import SoundWave from "./components/soundwave";


interface DispenseScreenProps{
    snack: Ingredient | undefined,
    mode: DispenseType
}

const DispenseScreen: React.FC<DispenseScreenProps> = ({snack, mode}) => {
    
    const classicModeOn = mode == DispenseType.Classic;
    const smallLargeModeOn = mode === DispenseType.LargeSmall;

    const [bowlCount, setBowlCount] = useState<Number>(0);
    const [peBlocked, setPeBlocked] = useState<Boolean>(false);
    const [dispenserBusy, setDispenserBusy] = useState<Boolean>(false);
    const [size, setSize] = useState<UiRequest>(UiRequest.None);
    const [timedOut, setTimedOut] = useState<Boolean>(false);

    const fetchIchibuState = async () => {
        try{
            const bowls = await invoke<Number>("get_dispense_count");
            setBowlCount(bowls);
            const peState = await invoke<Boolean>("get_pe_blocked");
            setPeBlocked(peState);
            const dispenserBusy = await invoke<Boolean>("dispenser_is_busy");
            setDispenserBusy(dispenserBusy);
            const timedOut = await invoke<Boolean>("dispenser_has_timed_out");
            setTimedOut(timedOut);
        } catch (error) {
            console.error("Failed to update bowl count: ", error);
        }
    }

    // Function to get the main button's text based on PE blocked state
    const getButtonText = () => {
        if(timedOut) {
            return 'Empty, please refill'
        }
        if(!peBlocked) {
            return 'Please place bowl in bay below';
        }
        if (classicModeOn){
            return 'Dispense';
        }
        return size !== UiRequest.None ? 'Dispense' : 'Please select a portion size'     
    };

    
    // Function to get the main button's class based on PE blocked state
    const getButtonClass = () => {
        const baseClasses = "w-full h-[150px] text-5xl font-bold focus:outline-none focus:ring-0 border-0";
        const readyClass = 'bg-green-600 hover:bg-green-700 active:bg-green-700';
        const notReadyClass = 'bg-gray-500 hover:bg-gray-500 active:bg-gray-500';
        const timedOutClass = "='bg-destructive hover:bg-destructive active:bg-destructive";

        if(timedOut) {
            return `${baseClasses} ${timedOutClass}`;
        }
        // Gray if PE NOT MADE
        if (!peBlocked) {
            return `${baseClasses} ${notReadyClass}`;
        }
        // Green if PE MADE and classing mode

        if(classicModeOn){
            return `${baseClasses} ${readyClass}`;
        }
        //Green if PE Made and Size has been selected, gray if not
        return size !== UiRequest.None ? `${baseClasses} ${readyClass}` : `${baseClasses} ${notReadyClass}`
    };

    const getSizeButtonClass = (buttonType: UiRequest) => {
        const baseClass = "w-1/2 h-[150px] text-6xl font-bold focus:outline-none focus:ring-0 border-0";
        const selectedColor = "bg-blue-600 hover:bg-blue-600";
        const gray = "bg-gray-500 hover:bg-gray-500 active:bg-gray-500";
        return size === buttonType ? `${baseClass} ${selectedColor}` : `${baseClass} ${gray}`;
        
    }

    const disableButton = () => {
        if (timedOut) {
            return true;
        }
        if (!peBlocked){
            return true;
        }
        if(classicModeOn) {
            return false;
        }
        return size !== UiRequest.None ? false : true;
    }

    const handleClick = async (size: UiRequest) => {
        try {
            if (classicModeOn) {
                size = UiRequest.RegularDispense
            }
            await invoke("update_ui_request", {uiRequest: size});
            console.log("Snack Selected updating state with: ", size);
        } catch(error){
            console.error("Failed to send state: ", error)
        }
    }

        useEffect(() => {
            if (!peBlocked) {
                setSize(UiRequest.None);
            }
        }, [peBlocked]);

 
        useEffect(() => {
            // Fetch immediately on component mount
            fetchIchibuState();
            
            // Set up interval for periodic updates (every 250 ms)
            const intervalId = setInterval(() => {
    
                fetchIchibuState();
            }, 500); // Adjust timing as needed
            
            // Clean up interval on component unmount
            return () => clearInterval(intervalId);
          }, []);


    return (
        <div className="px-10 py-44">
            
             <SvgViewer base64svg={snack?.base64_img ?? ""}/>
             <div className="w-full space-y-5 py-20">
                {
                    smallLargeModeOn &&
                    <div className="flex space-x-4 w-full">
                        <Button  
                            className={getSizeButtonClass(UiRequest.SmallDispense)}
                            onClick={() => setSize(UiRequest.SmallDispense)}
                        >
                            Small
                        </Button>
                        <Button  
                            className={getSizeButtonClass(UiRequest.RegularDispense)}
                            onClick={() => setSize(UiRequest.RegularDispense) }
                        >
                            Large
                        </Button>
                    </div>
                }

                    <Button  
                        disabled={disableButton()}
                        className={getButtonClass()}
                        onClick={() => handleClick(size)}
                    >
                        {getButtonText()}
                    </Button>
                    <div>
                        {/*<div className="flex items-center justify-center py-5 ">
                            {dispenserBusy && <span className="text-white text-2xl">Dispensing...</span>}
                        </div>*/}
                        <div className="flex items-center justify-center py-7">
                            {dispenserBusy && <SoundWave/>}
                        </div>
                    </div>
                </div>
                <div id="bag counter" className="absolute bottom-0 left-0 w-full flex items-center justify-center space-x-2 py-10">
                        <img 
                            src={recycle }
                            alt="Recycle" 
                            className="w-20 h-20 px-1"
                        />
                    <span className="text-white text-6xl">{bowlCount.toString()} plastic bags saved!</span>
                    </div>
        </div>
    )
}

export default DispenseScreen;

