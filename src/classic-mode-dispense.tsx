import React, { useEffect, useState } from "react";
import { Ingredient, UiRequest } from "./types";
import { Card, CardContent, CardFooter} from "./components/ui/card";
import SvgViewer from "./components/svg-viewer";
import { Button } from "./components/ui/button";
import recycle from "./assets/recycle.svg"
import { invoke } from "@tauri-apps/api/core";


interface ClassicModeDispenseProps {
    snack: Ingredient | undefined
}

const ClassicModeDispense: React.FC<ClassicModeDispenseProps> =({snack}) => {
    const [bowlCount, setBowlCount] = useState<Number>(0);
    const fetchBowlStatus = async () => {
        try{
            const bowls = await invoke<Number>("get_dispense_count");
            setBowlCount(bowls);
        } catch (error) {
            console.error("Failed to update bowl count: ", error);
        }
        
    }

    useEffect(() => {
        // Fetch immediately on component mount
        fetchBowlStatus();
        
        // Set up interval for periodic updates (every 2 seconds)
        const intervalId = setInterval(() => {
          fetchBowlStatus();
        }, 1000); // Adjust timing as needed
        
        // Clean up interval on component unmount
        return () => clearInterval(intervalId);
      }, []);

    const handleClick = async () => {
        try {
            await invoke("update_ui_request", {uiRequest: UiRequest.RegularDispense});
            console.log("Snack Selected updating state with: ", UiRequest.RegularDispense);
        } catch(error){
            console.error("Failed to send state: ", error)
        }
    }
    return (
        <div className="flex flex-col items-center justify-center h-full">
        <Card className="w-full max-w-md bg-slate-950">
            <CardContent className="flex flex-col items-center space-y-4 p-6">
                <div className="mb-4">
                    <SvgViewer base64svg={snack?.base64_img} height="h-96"/>
                </div>
                <div className="w-full space-y-2">
                    <Button  
                        className="w-full bg-green-600 hover:bg-green-700"
                        onClick={() => handleClick()}
                    >
                        Lets Ichibu!
                    </Button>
                </div>
            </CardContent>
            <CardFooter className="flex items-center justify-center p-4">
                    <div className="flex items-center space-x-2">
                        <img 
                            src={recycle }
                            alt="Recycle" 
                            className="w-6 h-6"
                        />
                    <span className="text-white">{bowlCount.toString()} plastic bags saved!</span>
                </div>
            </CardFooter>
        </Card>
    </div>
    )
}

export default ClassicModeDispense;