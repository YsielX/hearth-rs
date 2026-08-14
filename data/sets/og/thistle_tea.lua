local card={api_version=1,id="OG_073",name="Thistle Tea",text="Draw a card. Add 2 extra copies of it to your hand.",set="OG",type="spell",class="rogue",rarity="rare",cost=6}
function card.on_play(ctx,self)local p=ctx:controller(self);local deck=ctx:deck(p);ctx:draw(p,1);if #deck>0 then ctx:continue_with_entity("copy_drawn",deck[1])end end
function card.copy_drawn(ctx,self,entity)local p=ctx:controller(self);ctx:give_copy(p,entity);ctx:give_copy(p,entity)end
return card
