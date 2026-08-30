local card={api_version=1,id="ICC_289",name="Moorabi",text="Whenever another minion is <b>Frozen</b>, add a copy of it to your hand.",set="ICECROWN",type="minion",class="shaman",rarity="legendary",cost=6,attack=4,health=4}
card.triggers={{event="frozen",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.target~=self and ctx:entity(event.target).type=="minion"end,effect=function(ctx,self,event)cardlib.effects.give_base_copy(ctx, ctx:controller(self),event.target)end}}
return card
