import React from "react";
import { Ingredient } from "./types";
import SvgViewer from "./components/svg-viewer";
import { Card, CardContent, CardFooter } from "./components/ui/card";
import { Button } from "./components/ui/button";
import recycle from "./assets/recycle.svg"

interface SizedModeDispenseProps{
    snack: Ingredient | undefined
}
const SizedModeDispense: React.FC<SizedModeDispenseProps> =({snack}) => {
    return (
        <div className="flex flex-col items-center justify-center h-full">
        <Card className="w-full max-w-md bg-slate-950">
            <CardContent className="flex flex-col items-center space-y-4 p-6">
                <div className="mb-4">
                    <SvgViewer base64svg={snack?.base64_img} height="h-96"/>
                </div>
                <div className="w-full space-y-2">
                    <div className="flex space-x-2 w-full">
                        <Button  
                            className="w-1/2 bg-blue-600 hover:bg-blue-700"
                            onClick={() => console.log("Dispense a lil bit!")}
                        >
                            Just a tad!
                        </Button>
                        <Button  
                            className="w-1/2 bg-blue-600 hover:bg-blue-700"
                            onClick={() => console.log("Dispense a lot!")}
                        >
                            No ones looking!
                        </Button>
                    </div>
                    <Button  
                        className="w-full bg-green-600 hover:bg-green-700"
                        onClick={() => console.log("Dispense!")}
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
                    <span className="text-white">99 plastic bags saved</span>
                </div>
            </CardFooter>
        </Card>
    </div>
    )
}

export default SizedModeDispense;