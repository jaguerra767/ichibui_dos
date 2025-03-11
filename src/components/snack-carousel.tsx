import React from 'react';
import { Card, CardContent } from "./ui/card";
import { 
    Carousel, 
    CarouselContent, 
    CarouselItem, 
    CarouselNext, 
    CarouselPrevious 
} from "./ui/carousel";

import SvgViewer from './svg-viewer';
import { DispenseType, Ingredient, RunState, User } from '@/types';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';

interface SnackCarouselProps {
    dispenseType: DispenseType
    snacks: Ingredient[]
    setSnack: (snack: Ingredient) => void,
    setUser: (user: User) => void

}

const SnackCarousel: React.FC<SnackCarouselProps> = ({dispenseType, snacks, setSnack, setUser}) => {
    const navigate = useNavigate()
    const handleClick = async (snack: Ingredient) => {
        let state: RunState = RunState.Ready;
        try {
            if (dispenseType === DispenseType.Classic) {
                state = RunState.RunningClassic;
            } else if (dispenseType === DispenseType.LargeSmall) {
                state = RunState.RunningSized;
            }
            await invoke("update_run_state", {state})
        } catch(error){
            console.error("Failed to send state")
        }
        setSnack(snack);
        setUser(User.None)
        navigate('/dispense-screen');
    }
    return (
        <div className="w-full">
        <div className="mx-auto max-w-4xl">
            <Carousel 
                opts={{
                    align: "start", 
                    loop: false, 
                    slidesToScroll: 1
                }} 
                className="w-full" 
                orientation="vertical"
            >
                <CarouselContent className="h-[600px] space-y-4 py-10">
                    {snacks.map((snack, index) => (
                        <CarouselItem 
                            key={index} 
                            onClick={() => handleClick(snack)} 
                            className="basis-1/2"
                        >
                            <Card className="overflow-hidden w-fit mx-auto">
                                <CardContent className="flex items-center justify-center bg-slate-950">
                                    <SvgViewer base64svg={snack.base64_img} height="h-60"/>
                                </CardContent>
                            </Card>
                        </CarouselItem>
                    ))}
                </CarouselContent>
                <CarouselPrevious className="bg-white"/>
                <CarouselNext className="bg-white"/>
            </Carousel>
        </div>
    </div>
    )
}

export default SnackCarousel;