local card={api_version=1,id="EX1_339",name="Thoughtsteal",text="Copy 2 cards in your opponent's deck and add them to your hand.",set="EXPERT1",type="spell",class="priest",rarity="common",spell_school="shadow",cost=2}
function card.on_play(ctx,self)local deck=ctx:deck(ctx:opponent(ctx:controller(self)));if #deck>0 then ctx:random_entity(deck,"thought_first")end end
function card.thought_first(ctx,self,e)ctx:set_data(self,"thought_first",e);ctx:give_copy(ctx:controller(self),e);local r={};for _,x in ipairs(ctx:deck(ctx:opponent(ctx:controller(self))))do if x~=e then r[#r+1]=x end end;if #r>0 then ctx:random_entity(r,"thought_second")end end
function card.thought_second(ctx,self,e)ctx:give_copy(ctx:controller(self),e)end
return card
