local card={api_version=1,id="OG_101",name="Forbidden Shaping",text="Spend all your Mana. Summon a random minion that costs that much.",set="OG",type="spell",class="priest",rarity="epic",spell_school="shadow",cost=0}
function card.on_play(ctx,self)local p=ctx:controller(self);local amount=ctx:player(p).mana;ctx:spend_mana(p,amount);local pool={};for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if d.type=="minion" and d.cost==amount then pool[#pool+1]=id end end;if #pool>0 then ctx:random_value(pool,"summon_minion")end end
function card.summon_minion(ctx,self,id)ctx:summon(ctx:controller(self),id)end
return card
