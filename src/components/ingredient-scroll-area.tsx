
import { ScrollArea, Scrollbar } from "@radix-ui/react-scroll-area";
import { Separator } from "@/components/ui/separator"

import  SvgViewer from "./svg-viewer";


interface IngredientScrollAreaProps{
    images: string[]
}
const IngredientScrollArea: React.FC<IngredientScrollAreaProps> = ({images}) => {

    return (
        <ScrollArea className=" h-81 w-48 rounded-md border bg-slate-700 whitespace-nowrap overflow-y-hidden">
            <div className="p-4">
                <h4 className= "mb-4 text-lg text-slate-50 font-medium  leading-none"> Snacks </h4>
                {images.map((image: string, index: number) => (
                    <>
                        <div className="overflow-hidden">
                        <div key={index} className="text-sm text-slate-50">
                        <SvgViewer base64svg={image} height={"h-56"}></SvgViewer>
                        </div>
                        <Separator className="my-2" />
                        </div>
                    </>
                ))}
            </div>
            <Scrollbar orientation="vertical"/>
        </ScrollArea>
    );
};

export default IngredientScrollArea;