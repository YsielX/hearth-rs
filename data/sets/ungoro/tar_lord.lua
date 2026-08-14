local card={api_version=1,id="UNG_838",name="Tar Lord",text="<b>Taunt</b>\nHas +4 Attack during your opponent's turn.",set="UNGORO",type="minion",class="warrior",rarity="common",cost=6,attack=1,health=11,tags={"elemental"},keywords={"taunt"}}
card.auras={{targets=function(ctx,self) if ctx:active_player()~=ctx:controller(self) then return {self} end return {} end,attack=4}}
return card
