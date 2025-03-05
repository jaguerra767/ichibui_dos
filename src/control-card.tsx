import React from "react";
import { useNavigate } from "react-router-dom";

import { Card, CardContent, CardHeader, CardTitle } from "./components/ui/card";
import { Button } from "./components/ui/button";

import SvgViewer from "./components/svg-viewer";
import { User } from "./types";


interface ControlCardProps {
    image: string | undefined
    setUser: (user: User) => void
}

const ControlCard: React.FC<ControlCardProps> = ({image, setUser}) => {

    const navigate = useNavigate();

    return (
        <Card className="w-full h-full flex flex-col overflow-hidden bg-slate-950">
            <CardHeader>
                <CardTitle className="text-white">
                    Current Selection
                </CardTitle>
            </CardHeader>
            
            <CardContent className="h-1/2 flex flex-col justify-center p-4">
                <div className="h-1/2">
                    <SvgViewer base64svg={image} height="h-56"></SvgViewer>
                </div>
                <Button  
                className="w-full bg-green-600"
                onClick={() => {
                    setUser(User.Operator);
                    navigate("/dispense-screen");}}
                >Lets Ichibu!</Button>
            </CardContent> 
        </Card>
    );
};

export default ControlCard;

