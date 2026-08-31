local card = {
    api_version = 1, id = "AT_037", name = "Living Roots",
    text = "<b>Choose One -</b> Deal $2 damage; or Summon two 1/1 Saplings.",
    set = "TGT", type = "spell", class = "druid", rarity = "common",
    spell_school = "nature", cost = 1, keywords = { "choose_one" },
}

local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == wanted then return true end
    end
    return false
end

local function damage_targets(ctx, self)
    local player = ctx:controller(self)
    local result = {}
    for _, character in ipairs(ctx:characters()) do
        local entity = ctx:entity(character)
        local legal = not has_keyword(ctx, character, "dormant")
        if legal and entity.controller ~= player then
            legal = not has_keyword(ctx, character, "stealth")
                and not has_keyword(ctx, character, "elusive")
                and not has_keyword(ctx, character, "immune")
        end
        if legal then result[#result + 1] = character end
    end
    return result
end

local function summon_saplings(ctx, self)
    local player = ctx:controller(self)
    ctx:summon(player, "AT_037t")
    ctx:summon(player, "AT_037t")
end

local function choose_damage_target(ctx, self)
    ctx:choose_entities(ctx:controller(self), "Deal 2 damage", damage_targets(ctx, self), "deal_damage")
end

function card.on_choose_one(ctx, self)
    local options = { { card_id = "AT_037a", label = "Deal 2 damage" } }
    if #ctx:board(ctx:controller(self)) < 7 then
        options[#options + 1] = { card_id = "AT_037b", label = "Summon two 1/1 Saplings" }
    end
    if #options == 1 then card.chosen(ctx, self, options[1].card_id)
    else ctx:choose_options(ctx:controller(self), "Choose One", options, "chosen") end
end

function card.chosen(ctx, self, choice)
    if choice == "AT_037a" then choose_damage_target(ctx, self) else summon_saplings(ctx, self) end
end

function card.deal_damage(ctx, self, target) cardlib.effects.damage(ctx, target, 2) end

function card.on_choose_multiple(ctx, self)
    choose_damage_target(ctx, self)
    if #ctx:board(ctx:controller(self)) < 7 then summon_saplings(ctx, self) end
end

card.tokens = {
    { id = "AT_037a", spell_school = "nature", name = "Grasping Roots", text = "Deal $2 damage.", set = "TGT", type = "spell", class = "druid", collectible = false, cost = 1 },
    { id = "AT_037b", spell_school = "nature", name = "One, Two, Trees!", text = "Summon two 1/1 Saplings.", set = "TGT", type = "spell", class = "druid", collectible = false, cost = 1 },
    { id = "AT_037t", name = "Sapling", text = "", set = "TGT", type = "minion",
      class = "druid", cost = 1, attack = 1, health = 1 },
}

return card
