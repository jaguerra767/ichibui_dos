import React from 'react';
import{ Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch"
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from "@/components/ui/dropdown-menu";

import gear from '@/assets/gear-white.svg';
import { Label } from '@radix-ui/react-dropdown-menu';


import { DispenseType, User } from '@/types';

interface SettingsMenuProps{
    currentUser: User,
    currentDispenseType: DispenseType
    setDispenseType: (type: DispenseType) => void;
}

const SettingsMenu: React.FC<SettingsMenuProps> = ({currentUser, currentDispenseType, setDispenseType}) => {
    const superVisibility = currentUser === User.Admin || currentUser === User.Manager;
    const handleToggle = (checked: Boolean) => {
        setDispenseType(checked ? DispenseType.LargeSmall : DispenseType.Classic);
        console.log(checked ? DispenseType.LargeSmall : DispenseType.Classic)
    }
    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button className='bg-slate-950'> 
                    <img src={gear} alt='settings' className='h-10'></img>
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className='bg-gray-600'>
                {superVisibility && <DropdownMenuItem onClick={() => console.log('Option 1 clicked')}>
                    <Switch 
                        checked={currentDispenseType === DispenseType.LargeSmall}
                        onCheckedChange={handleToggle}
                        id="dispense-mode"/>
                    <Label>Custom Dispense</Label>
                </DropdownMenuItem>}
                <DropdownMenuItem onClick={() => console.log('Option 2 clicked')}>
                    <Button className='w-full'>
                        Clean Mode
                    </Button>
                </DropdownMenuItem>
                <DropdownMenuItem>
                <Button className='w-full bg-destructive'>
                        Empty Hopper
                    </Button>
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    )
}

export default SettingsMenu;

