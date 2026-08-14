local function deathrattle(def)for _,k in ipairs(def.keywords or{})do if k=="deathrattle"then return true end end return false end
local card={api_version=1,id="LOOT_520",name="Seeping Oozeling",text="<b>Battlecry:</b> Gain the <b>Deathrattle</b> of a random minion in your deck.",set="LOOTAPALOOZA",type="minion",class="hunter",rarity="rare",cost=6,attack=5,health=4,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local pool={};for _,e in ipairs(ctx:deck(ctx:controller(self)))do local d=ctx:card_definition(ctx:entity(e).card_id);if d.type=="minion"and deathrattle(d)then pool[#pool+1]=d.id end end;if #pool>0 then ctx:random_value(pool,"oozeling_deathrattle")end end
function card.oozeling_deathrattle(ctx,self,id)ctx:attach_deathrattle(self,id);ctx:grant_keyword(self,"deathrattle")end
return card
