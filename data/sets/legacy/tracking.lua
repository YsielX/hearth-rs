local card={api_version=1,id="DS1_184",name="Tracking",text="<b>Discover</b> a card from your deck.",set="LEGACY",type="spell",class="hunter",cost=1,keywords={"discover"}}
function card.on_play(ctx,self)local p=ctx:controller(self);local deck=ctx:deck(p);if #deck>0 then ctx:discover_entities(p,"Discover a card from your deck",deck,3,"draw_selected")end end
function card.draw_selected(ctx,self,target)ctx:draw_entity(ctx:controller(self),target)end
return card
