local card={api_version=1,id="UNG_049",name="Tar Lurker",text="<b>Taunt</b>\nHas +3 Attack during your opponent's turn.",set="UNGORO",type="minion",class="warlock",rarity="common",cost=5,attack=1,health=7,tags={"elemental"},keywords={"taunt"}}
card.auras={{targets=function(ctx,self) if ctx:active_player()~=ctx:controller(self) then return {self} end return {} end,attack=3}}
return card
