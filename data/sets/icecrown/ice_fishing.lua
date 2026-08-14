local card={api_version=1,id="ICC_089",name="Ice Fishing",text="Draw 2 Murlocs from your deck.",set="ICECROWN",type="spell",class="shaman",rarity="common",spell_school="frost",cost=2}
local function murloc(ctx,e)for _,t in ipairs(ctx:card_definition(ctx:entity(e).card_id).tags or{})do if t=="murloc"or t=="all"then return true end end return false end
function card.on_play(ctx,self)ctx:set_data(self,"draws",2);ctx:continue_with("draw_murloc")end
function card.draw_murloc(ctx,self)local pool={}for _,e in ipairs(ctx:deck(ctx:controller(self)))do if murloc(ctx,e)then pool[#pool+1]=e end end;if #pool>0 and ctx:get_data(self,"draws")>0 then ctx:random_entity(pool,"draw_selected_murloc")end end
function card.draw_selected_murloc(ctx,self,e)ctx:draw_entity(ctx:controller(self),e);local n=ctx:get_data(self,"draws")-1;ctx:set_data(self,"draws",n);if n>0 then ctx:continue_with("draw_murloc")end end
return card
