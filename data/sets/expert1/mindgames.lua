local card={api_version=1,id="EX1_345",name="Mindgames",text="Put a copy of\na random minion from\nyour opponent's deck into the battlefield.",set="EXPERT1",type="spell",class="priest",rarity="epic",spell_school="shadow",cost=4}
function card.on_play(ctx,self)local r={};for _,e in ipairs(ctx:deck(ctx:opponent(ctx:controller(self))))do if ctx:entity(e).type=="minion"then r[#r+1]=e end end;if #r>0 then ctx:random_entity(r,"mindgames_minion")end end
function card.mindgames_minion(ctx,self,e)ctx:summon(ctx:controller(self),ctx:entity(e).card_id)end
return card
