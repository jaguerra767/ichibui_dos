import React from 'react';
import { Card, CardContent } from "./ui/card";
import { 
    Carousel, 
    CarouselContent, 
    CarouselItem, 
    CarouselNext, 
    CarouselPrevious 
} from "./ui/carousel";


import { DispenseType, Ingredient, IchibuState, User } from '@/types';
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
        let state: IchibuState = IchibuState.Ready;
        try {

            await invoke("update_current_ingredient", {snack: snack.id})
            if (dispenseType === DispenseType.Classic) {
                state = IchibuState.RunningClassic;
            } else if (dispenseType === DispenseType.LargeSmall) {
                state = IchibuState.RunningSized;
            }
            console.log("Snack Selected updating state with: ", state, snack);
            await invoke("update_run_state", {newState: state})
        } catch(error){
            console.error("Failed to send state: ", error)
        }
        setSnack(snack);
        setUser(User.None)
        navigate('/dispense-screen');
    }
    return (
        <div className="w-full">
        <div className="mx-auto max-w-5xl">
            <Carousel 
                opts={{
                    align: "start", 
                    loop: false, 
                    slidesToScroll: 1
                }} 
                className="w-full" 
                orientation="vertical"
            >
                <CarouselContent className="h-[1300px] space-y-1 py-1">
                    {snacks.map((snack, index) => (
                        <CarouselItem 
                            key={index} 
        
                            onClick={() => handleClick(snack)} 
                            className="basis-1/2"
                        >
                            <Card className="overflow-hidden w-fit mx-auto">
                                <CardContent className="flex items-center justify-center bg-slate-950">
                                    <img src={snack.base64_img} alt="snack" className='h-[600px] w-full'></img>
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