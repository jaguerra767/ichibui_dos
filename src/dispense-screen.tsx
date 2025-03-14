import React, { useEffect, useState } from "react";
import { DispenseType, Ingredient, UiRequest} from "./types";
import recycle from "./assets/recycle.svg"
import { invoke } from "@tauri-apps/api/core";
import SvgViewer from "./components/svg-viewer";
import { Button } from "./components/ui/button";


interface DispenseScreenProps{
    snack: Ingredient | undefined,
    mode: DispenseType
}

const DispenseScreen: React.FC<DispenseScreenProps> = ({snack, mode}) => {
    
    const classicModeOn = mode == DispenseType.Classic;
    const smallLargeModeOn = mode === DispenseType.LargeSmall;

    const [bowlCount, setBowlCount] = useState<Number>(0);
    const [peBlocked, setPeBlocked] = useState<Boolean>(false);
    const [size, setSize] = useState<UiRequest>(UiRequest.None);

    const fetchIchibuState = async () => {
        try{
            const bowls = await invoke<Number>("get_dispense_count");
            setBowlCount(bowls);
            const peState = await invoke<Boolean>("get_pe_blocked")
            setPeBlocked(peState);
        } catch (error) {
            console.error("Failed to update bowl count: ", error);
        }
    }

    // Function to get the main button's text based on PE blocked state
    const getButtonText = () => {
        if(!peBlocked) {
            return 'Plase place bowl in bay below';
        }
        if (classicModeOn){
            return 'Lets Go!';
        }
        return size !== UiRequest.None ? 'lets Go!' : 'Please select a portion size'     
    };

    
    // Function to get the main button's class based on PE blocked state
    const getButtonClass = () => {
        const baseClasses = "w-full h-[120px] text-4xl";
        const readyClass = 'bg-green-600 hover:bg-green-700 active:bg-green-800';
        const notReadyClass = 'bg-gray-500 hover:bg-gray-500 active:bg-gray-500';
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

    const disableButton = () => {
        if (!peBlocked){
            return true
        }
        if(classicModeOn) {
            return false
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
            // Fetch immediately on component mount
            fetchIchibuState();
            
            // Set up interval for periodic updates (every 2 seconds)
            const intervalId = setInterval(() => {
              fetchIchibuState();
            }, 250); // Adjust timing as needed
            
            // Clean up interval on component unmount
            return () => clearInterval(intervalId);
          }, []);


    return (
        <div className="px-10 py-56">
             <SvgViewer base64svg={snack?.base64_img ?? ""}/>
             <div className="w-full space-y-2">
                {
                    smallLargeModeOn &&
                    <div className="flex space-x-2 w-full">
                        <Button  
                            className="w-1/2 h-[100px] text-2xl bg-blue-600 hover:bg-blue-950"
                            onClick={() => setSize(UiRequest.SmallDispense)}
                        >
                            SM
                        </Button>
                        <Button  
                            className="w-1/2 h-[100px]  text-2xl bg-blue-600 hover:bg-blue-950"
                            onClick={() => setSize(UiRequest.RegularDispense) }
                        >
                            LG
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
                    <div className="flex items-center justify-center space-x-2 py-96">
                        <img 
                            src={recycle }
                            alt="Recycle" 
                            className="w-20 h-20 px-1"
                        />
                    <span className="text-white text-6xl">{bowlCount.toString()} plastic bags saved!</span>
                </div>
                </div>
        </div>
    )
}

export default DispenseScreen;

