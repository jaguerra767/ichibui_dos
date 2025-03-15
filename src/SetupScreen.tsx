import React from 'react';
import { DispenseType, Ingredient, User } from './types';
import SnackCarousel from './components/snack-carousel';


interface SetupScreenProps{
    dispenseType: DispenseType,
    snacks: Ingredient[],
    setIngredient: (snack: Ingredient) => void,
    setUser: (user: User) => void
}

const SetupScreen: React.FC<SetupScreenProps> = ({dispenseType, snacks, setIngredient, setUser}) => {
   
    return (
        <div className='flex items-center justify-center h-screen'>
                <SnackCarousel dispenseType={dispenseType} snacks={snacks} setSnack={setIngredient} setUser={setUser}/>
        </div>
        
  
    );
};

export default SetupScreen;
