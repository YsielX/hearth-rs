local card={api_version=1,id="NEW1_008",name="Ancient of Lore",text="<b>Choose One -</b> Draw 2 cards; or Restore #7 Health.",set="EXPERT1",type="minion",class="druid",rarity="epic",cost=7,attack=7,health=7,keywords={"choose_one"}}
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{label="Draw 2 cards",value=1},{label="Restore 7 Health",value=2}},"chosen")end
function card.chosen(ctx,self,c)if c==1 then ctx:draw(ctx:controller(self),2)else ctx:choose_entities(ctx:controller(self),"Choose a character",ctx:characters(),"heal_selected")end end
function card.heal_selected(ctx,self,target)cardlib.effects.heal(ctx, target,7)end
function card.on_choose_multiple(ctx,self)ctx:draw(ctx:controller(self),2);ctx:choose_entities(ctx:controller(self),"Choose a character",ctx:characters(),"heal_selected")end
return card
