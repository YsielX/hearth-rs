local card = { api_version=1,id="OG_006",name="Vilefin Inquisitor",text="<b>Battlecry:</b> Your Hero Power becomes 'Summon a   1/1 Murloc.'",set="OG",type="minion",class="paladin",rarity="epic",cost=1,attack=1,health=3,tags={"murloc"},keywords={"battlecry"} }
function card.on_battlecry(ctx,self) ctx:replace_hero_power(ctx:controller(self),"OG_006b") end
card.tokens={
 {id="OG_006a",name="Silver Hand Murloc",text="",set="OG",type="minion",class="paladin",cost=1,attack=1,health=1,tags={"murloc"}},
 {id="OG_006b",name="The Tidal Hand",text="Summon a 1/1 Silver Hand Murloc.",set="OG",type="hero_power",class="paladin",cost=2,on_play=function(ctx,self) ctx:summon(ctx:controller(self),"OG_006a") end},
}
return card
