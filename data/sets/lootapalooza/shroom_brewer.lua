local card={api_version=1,id="LOOT_291",name="Shroom Brewer",text="<b>Battlecry:</b> Restore #4 Health.",set="LOOTAPALOOZA",type="minion",rarity="common",cost=4,attack=4,health=4,keywords={"battlecry"},target_mode="required",targets=function(ctx)return ctx:characters()end}
function card.on_battlecry(ctx,self,target)ctx:heal(target,4)end
return card
