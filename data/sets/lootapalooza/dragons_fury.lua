local card={api_version=1,id="LOOT_172",name="Dragon's Fury",text="Reveal a spell from your deck. Deal damage equal to its Cost to all minions.",set="LOOTAPALOOZA",type="spell",class="mage",rarity="epic",spell_school="fire",cost=5}
function card.on_play(ctx,self)local pool={};for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="spell"then pool[#pool+1]=e end end;if #pool>0 then ctx:random_entity(pool,"fury_spell")end end
function card.fury_spell(ctx,self,e)ctx:damage_all(ctx:minions(),ctx:entity(e).cost)end;return card
