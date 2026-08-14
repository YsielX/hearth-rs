local card={api_version=1,id="LOOT_085",name="Rhok'delar",text="<b>Battlecry:</b> If your deck has no minions, fill your hand with Hunter spells.",set="LOOTAPALOOZA",type="weapon",class="hunter",rarity="legendary",cost=7,attack=4,health=2,keywords={"battlecry"}}
local function fill(ctx,self)local p=ctx:controller(self);if #ctx:hand(p)>=10 then return end;local pool={};for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if d.type=="spell"and d.class=="hunter"then pool[#pool+1]=id end end;if #pool>0 then ctx:random_value(pool,"rhokdelar_spell")end end
function card.on_battlecry(ctx,self)for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="minion"then return end end;fill(ctx,self)end
function card.rhokdelar_spell(ctx,self,id)ctx:give_card(ctx:controller(self),id);ctx:continue_with("rhokdelar_continue")end
function card.rhokdelar_continue(ctx,self)fill(ctx,self)end
return card
