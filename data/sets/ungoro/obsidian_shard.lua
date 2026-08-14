local function is_non_rogue_class_card(definition)
    if definition.classes and #definition.classes > 0 then
        for _, class in ipairs(definition.classes) do if class == "rogue" then return false end end
        return true
    end
    return definition.class ~= "neutral" and definition.class ~= "rogue"
end
return {
    api_version = 1, id = "UNG_061", name = "Obsidian Shard",
    text = "Costs (1) less for each non-Rogue Class card added to your hand this game.",
    set = "UNGORO", type = "weapon", class = "rogue", rarity = "rare",
    cost = 4, attack = 3, health = 3,
    auras = {{
        active_zones = { "hand" },
        cost = function(ctx, self)
            local reduction = 0
            for _, id in ipairs(ctx:cards_added_to_hand(ctx:controller(self))) do
                if is_non_rogue_class_card(ctx:card_definition(id)) then reduction = reduction + 1 end
            end
            return -reduction
        end,
        targets = function(ctx, self) return { self } end,
    }},
}
