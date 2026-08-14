local card={api_version=1,id="UNG_938",name="Hot Spring Guardian",text="<b>Taunt</b>\n<b>Battlecry:</b> Restore #3 Health.",set="UNGORO",type="minion",class="shaman",rarity="common",cost=3,attack=2,health=4,tags={"elemental"},keywords={"taunt","battlecry"},target_mode="required_if_available"}
function card.targets(ctx,self) return ctx:characters() end
function card.on_battlecry(ctx,self,target) if target then ctx:heal(target,3) end end
return card
