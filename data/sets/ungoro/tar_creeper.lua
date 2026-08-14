local card={api_version=1,id="UNG_928",name="Tar Creeper",text="<b>Taunt</b>\nHas +2 Attack during your opponent's turn.",set="UNGORO",type="minion",class="neutral",rarity="common",cost=3,attack=1,health=5,tags={"elemental"},keywords={"taunt"}}
card.auras={{targets=function(ctx,self) if ctx:active_player()~=ctx:controller(self) then return {self} end return {} end,attack=2}}
return card
