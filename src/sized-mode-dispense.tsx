import React, { useEffect, useState } from "react";
import { Ingredient, UiRequest } from "./types";
import SvgViewer from "./components/svg-viewer";
import { Card, CardContent, CardFooter } from "./components/ui/card";
import { Button } from "./components/ui/button";
import recycle from "./assets/recycle.svg"
import { invoke } from "@tauri-apps/api/core";

interface SizedModeDispenseProps{
    snack: Ingredient | undefined
}
const SizedModeDispense: React.FC<SizedModeDispenseProps> =({snack}) => {
    const [bowlCount, setBowlCount] = useState<Number>(0);
    const [size, setSize] = useState<UiRequest>(UiRequest.None);
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

    const handleClick = async (size: UiRequest) => {
        try {
            await invoke("update_ui_request", {uiRequest: size});
            console.log("Snack Selected updating state with: ", size);
        } catch(error){
            console.error("Failed to send state: ", error)
        }
    }
    return (
        <div className="flex flex-col items-center justify-center h-full">
        <Card className="w-full max-w-lg bg-slate-950">
            <CardContent className="flex flex-col items-center space-y-4 p-6">
                <div className="mb-4">
                    <SvgViewer base64svg={snack?.base64_img} height="h-96"/>
                </div>
                <div className="w-full space-y-2">
                    <div className="flex space-x-2 w-full">
                        <Button  
                            className="w-1/2 h-[100px] text-2xl bg-blue-600 hover:bg-blue-700"
                            onClick={() => setSize(UiRequest.SmallDispense)}
                        >
                            Just a tad!
                        </Button>
                        <Button  
                            className="w-1/2 h-[100px]  text-2xl bg-blue-600 hover:bg-blue-700"
                            onClick={() => setSize(UiRequest.RegularDispense) }
                        >
                            Regular!
                        </Button>
                    </div>
                    <Button  
                        className="w-full h-[120px]  text-4xl bg-green-600 hover:bg-green-700"
                        onClick={() => handleClick(size)}
                    >
                        Lets Go!
                    </Button>
                </div>
            </CardContent>
            <CardFooter className="flex h-[100px] items-center justify-center p-4">
                    <div className="flex items-center space-x-2">
                        <img 
                            src={recycle }
                            alt="Recycle" 
                            className="w-10 h-10"
                        />
                    <span className="text-white text-2xl">{bowlCount.toString()} plastic bags saved!</span>
                </div>
            </CardFooter>
        </Card>
    </div>
    )
}

export default SizedModeDispense;

