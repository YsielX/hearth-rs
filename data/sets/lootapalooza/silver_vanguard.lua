local card={api_version=1,id="LOOT_184",name="Silver Vanguard",text="<b>Deathrattle:</b> <b>Recruit</b> an\n8-Cost minion.",set="LOOTAPALOOZA",type="minion",rarity="common",cost=7,attack=3,health=3,keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)local p={};for _,e in ipairs(ctx:deck(ctx:controller(self)))do local v=ctx:entity(e);if v.type=="minion"and v.cost==8 then p[#p+1]=e end end;if #p>0 then ctx:random_entity(p,"recruit_vanguard")end end
function card.recruit_vanguard(ctx,self,e)ctx:recruit(ctx:controller(self),e)end
return card
