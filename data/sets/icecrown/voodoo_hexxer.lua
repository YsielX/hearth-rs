local card={api_version=1,id="ICC_088",name="Voodoo Hexxer",text="<b>Taunt</b>\n<b>Freeze</b> any character damaged by this minion.",set="ICECROWN",type="minion",class="shaman",rarity="rare",cost=5,attack=2,health=7,keywords={"taunt"}}
card.triggers={{event="damaged",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.source==self and event.amount>0 end,effect=function(ctx,self,event)ctx:freeze(event.target)end}}
return card
